#!/usr/bin/env python3
"""Comprehensive Unit and Integration Tests for Privilege MCP Tools (T-02535/T-02536).

Covers all 5 Privilege Escalation Prevention tools over MCP JSON-RPC 2.0 stdio:
1. `aios.privilege.status`: Query actor context, tier, and capabilities.
2. `aios.privilege.elevate`: Dynamic elevation, grant gating, and SystemKernel immutability.
3. `aios.privilege.drop`: Safe voluntary de-escalation.
4. `aios.privilege.revoke`: Restoration of baseline privileges.
5. `aios.privilege.check`: Direct capability checking.
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


def run_mcp(payload: dict, env: dict | None = None, timeout_s: int = 30) -> dict:
    run_env = os.environ.copy()
    if env:
        run_env.update(env)
    p = subprocess.Popen(
        [get_mcp_binary()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        env=run_env,
    )
    try:
        stdout, _ = p.communicate(json.dumps(payload) + "\n", timeout=timeout_s)
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
        raise TimeoutError(f"aiosh-mcp timed out after {timeout_s}s")
    finally:
        if p.poll() is None:
            p.kill()
            p.wait()

    assert p.returncode == 0, f"aiosh-mcp exited with returncode {p.returncode}"
    return json.loads(stdout.strip())


def call_mcp_tool(tool_name: str, arguments: dict | None = None, env: dict | None = None, timeout_s: int = 30) -> dict:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments or {}},
    }
    resp = run_mcp(payload, env=env, timeout_s=timeout_s)
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


def list_mcp_tools(env: dict | None = None, timeout_s: int = 30) -> list[dict]:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {},
    }
    resp = run_mcp(payload, env=env, timeout_s=timeout_s)
    return resp.get("result", {}).get("tools", [])


def test_mcp_privilege_tool_registration():
    print("Testing tool registration...")
    tools = list_mcp_tools()
    tool_names = {t["name"] for t in tools}
    assert "aios.privilege.status" in tool_names, "missing aios.privilege.status"
    assert "aios.privilege.elevate" in tool_names, "missing aios.privilege.elevate"
    assert "aios.privilege.drop" in tool_names, "missing aios.privilege.drop"
    assert "aios.privilege.revoke" in tool_names, "missing aios.privilege.revoke"
    assert "aios.privilege.check" in tool_names, "missing aios.privilege.check"
    print("  Registration OK.")


def test_mcp_privilege_lifecycle():
    print("Testing privilege lifecycle over MCP...")
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "priv_store.json")
        env = {"AIOS_PRIVILEGE_STORE": store_path}

        # 1. Initial status
        res = call_mcp_tool("aios.privilege.status", {"actor": "agent-007"}, env=env)
        assert res.get("ok") is True, f"status failed: {res}"
        ctx = res["context"]
        assert ctx["actor"] == "agent-007"
        assert ctx["tier"] == "User"
        assert ctx["elevated"] is False
        assert ctx["grant_id"] is None

        # 2. Elevation without grant to Admin must fail
        res = call_mcp_tool("aios.privilege.elevate", {"to": "admin", "actor": "agent-007"}, env=env)
        assert res.get("ok") is False or "error" in res
        err_msg = str(res)
        assert "ERR_PRIVESC_GRANT_REQUIRED" in err_msg or "grant" in err_msg.lower()

        # 3. Elevation to SystemKernel must fail-fast
        res = call_mcp_tool("aios.privilege.elevate", {
            "to": "system_kernel",
            "grant": "grant-kernel",
            "actor": "agent-007",
        }, env=env)
        assert res.get("ok") is False or "error" in res
        assert "ERR_PRIVESC_KERNEL_TIER_IMMUTABLE" in str(res)

        # 4. Valid elevation with grant to Operator
        res = call_mcp_tool("aios.privilege.elevate", {
            "to": "operator",
            "grant": "grant-operator-123",
            "actor": "agent-007",
            "caps": ["network_listen", "filesystem_write"],
        }, env=env)
        assert res.get("ok") is True, f"elevation failed: {res}"
        ctx = res["context"]
        assert ctx["tier"] == "Operator"
        assert ctx["elevated"] is True
        assert ctx["grant_id"] == "grant-operator-123"

        # 5. Check capability
        res = call_mcp_tool("aios.privilege.check", {
            "cap": "network_listen",
            "actor": "agent-007",
        }, env=env)
        assert res.get("ok") is True
        assert res.get("held") is True

        res = call_mcp_tool("aios.privilege.check", {
            "cap": "system_reboot",
            "actor": "agent-007",
        }, env=env)
        assert res.get("ok") is True
        assert res.get("held") is False

        # 6. Drop privilege back to user
        res = call_mcp_tool("aios.privilege.drop", {"to": "user", "actor": "agent-007"}, env=env)
        assert res.get("ok") is True
        assert res["context"]["tier"] == "User"

        # 7. Revoke elevation
        res = call_mcp_tool("aios.privilege.revoke", {"actor": "agent-007"}, env=env)
        assert res.get("ok") is True
        assert res["context"]["tier"] == "User"
        assert res["context"]["elevated"] is False
    print("  Privilege lifecycle OK.")


def test_mcp_privilege_hardening():
    print("Testing input boundary hardening...")
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "priv_store.json")
        env = {"AIOS_PRIVILEGE_STORE": store_path}

        # Long actor ID (> 128 chars)
        bad_actor = "A" * 150
        res = call_mcp_tool("aios.privilege.status", {"actor": bad_actor}, env=env)
        assert res.get("ok") is False or "error" in res
        assert "ERR_PRIVESC_INVALID_INPUT" in str(res)

        # Control characters in actor
        ctrl_actor = "agent\x00injected"
        res = call_mcp_tool("aios.privilege.status", {"actor": ctrl_actor}, env=env)
        assert res.get("ok") is False or "error" in res
        assert "ERR_PRIVESC_INVALID_INPUT" in str(res)
    print("  Input boundary hardening OK.")


def test_mcp_privilege_config():
    print("Testing privilege config MCP tool...")
    res = call_mcp_tool("aios.privilege.config")
    assert res.get("ok") is True, f"privilege config failed: {res}"
    cfg = res["config"]
    assert cfg["version"] == "1.0.0"
    assert cfg["default_tier"] == "user"
    assert cfg["max_active_contexts"] == 1024
    assert cfg["max_capabilities_per_context"] == 32
    print("  Privilege config MCP OK.")


def main():
    print("=== Running Privilege MCP Tests ===")
    test_mcp_privilege_tool_registration()
    test_mcp_privilege_lifecycle()
    test_mcp_privilege_hardening()
    test_mcp_privilege_config()
    print("=== ALL PRIVILEGE MCP TESTS PASSED ===")


if __name__ == "__main__":
    main()

