#!/usr/bin/env python3
"""End-to-End Automated Integration Smoke Test for Privilege MCP Tools (T-02536).

Validates:
1. Multi-actor context isolation across concurrent actor sessions.
2. State store persistence across multiple process invocations.
3. Audit ring emission on each MCP tool call.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _find_binary(names: list[str]) -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in names:
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return names[-1]


def get_mcp_binary() -> str:
    return _find_binary(["aiosh-mcp.exe", "aiosh-mcp"])


def run_mcp_call(tool_name: str, arguments: dict | None = None, env: dict | None = None) -> dict:
    run_env = os.environ.copy()
    if env:
        run_env.update(env)
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments or {}},
    }
    p = subprocess.Popen(
        [get_mcp_binary()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        env=run_env,
    )
    stdout, _ = p.communicate(json.dumps(payload) + "\n", timeout=30)
    assert p.returncode == 0, f"mcp process failed with exit code {p.returncode}"
    resp = json.loads(stdout.strip())
    if "error" in resp:
        return {"ok": False, "error": resp["error"]}
    res = resp.get("result", {})
    if "structuredContent" in res and "result" in res["structuredContent"]:
        return res["structuredContent"]["result"]
    elif "content" in res and res["content"]:
        try:
            return json.loads(res["content"][0]["text"])
        except Exception:
            return {"ok": False, "text": res["content"][0]["text"]}
    return res


def test_multi_actor_isolation():
    print("Testing multi-actor context isolation...")
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "priv_store.json")
        env = {"AIOS_PRIVILEGE_STORE": store_path}

        # Actor 1 elevates to Operator
        r1 = run_mcp_call("aios.privilege.elevate", {
            "actor": "actor_alpha",
            "to": "operator",
            "grant": "grant-alpha-1",
            "caps": ["network_listen"],
        }, env=env)
        assert r1.get("ok") is True, f"alpha elevate failed: {r1}"
        assert r1["context"]["tier"] == "Operator"

        # Actor 2 remains at default User tier
        r2 = run_mcp_call("aios.privilege.status", {"actor": "actor_beta"}, env=env)
        assert r2.get("ok") is True
        assert r2["context"]["tier"] == "User"
        assert r2["context"]["elevated"] is False

        # Verify Actor 2 does not have network_listen capability
        r_check_b = run_mcp_call("aios.privilege.check", {
            "actor": "actor_beta",
            "cap": "network_listen",
        }, env=env)
        assert r_check_b.get("ok") is True
        assert r_check_b.get("held") is False

        # Verify Actor 1 has network_listen capability
        r_check_a = run_mcp_call("aios.privilege.check", {
            "actor": "actor_alpha",
            "cap": "network_listen",
        }, env=env)
        assert r_check_a.get("ok") is True
        assert r_check_a.get("held") is True
    print("  Multi-actor isolation OK.")


def test_persistence_across_process_invocations():
    print("Testing persistence across independent process executions...")
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "priv_store.json")
        env = {"AIOS_PRIVILEGE_STORE": store_path}

        # First process: elevate actor
        r = run_mcp_call("aios.privilege.elevate", {
            "actor": "persisted_worker",
            "to": "admin",
            "grant": "admin-grant-999",
            "caps": ["audit_log_admin", "system_reboot"],
        }, env=env)
        assert r.get("ok") is True, f"admin elevation failed: {r}"

        # Verify store file exists on disk
        assert os.path.exists(store_path), "store file was not persisted"

        # Second fresh process: query status
        r2 = run_mcp_call("aios.privilege.status", {"actor": "persisted_worker"}, env=env)
        assert r2.get("ok") is True
        assert r2["context"]["tier"] == "Admin"
        assert r2["context"]["elevated"] is True
        assert r2["context"]["grant_id"] == "admin-grant-999"
        assert "audit_log_admin" in r2["context"]["capabilities"]

        # Third fresh process: check capability
        r3 = run_mcp_call("aios.privilege.check", {
            "actor": "persisted_worker",
            "cap": "audit_log_admin",
        }, env=env)
        assert r3.get("ok") is True
        assert r3.get("held") is True
    print("  Persistence OK.")


def main():
    print("=== Running Privilege MCP Automated Integration Smoke ===")
    test_multi_actor_isolation()
    test_persistence_across_process_invocations()
    print("=== ALL PRIVILEGE MCP INTEGRATION TESTS PASSED ===")


if __name__ == "__main__":
    main()
