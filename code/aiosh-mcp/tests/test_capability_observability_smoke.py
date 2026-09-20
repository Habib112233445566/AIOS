#!/usr/bin/env python3
"""Integration Smoke Test for Capability Observability via MCP (T-02076).

Validates end-to-end observability on the MCP JSON-RPC boundary:
1. Tool registration: `aios.capability.observability` present in `tools/list`.
2. Report generation on empty store.
3. Accurate metric updates across issuance, attenuation, and cascade revocation.
4. Health evaluation and JSON serialization parity.
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


def run_mcp(payload: dict, timeout_s: int = 30) -> dict:
    p = subprocess.Popen(
        [get_mcp_binary()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
    )
    try:
        stdout, _ = p.communicate(json.dumps(payload) + "\n", timeout=timeout_s)
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
        print(f"FAIL: aiosh-mcp timed out after {timeout_s}s")
        sys.exit(1)
    finally:
        if p.poll() is None:
            p.kill()
            p.wait()

    if p.returncode != 0:
        print(f"FAIL: aiosh-mcp returned {p.returncode}")
        sys.exit(1)
    try:
        return json.loads(stdout.strip())
    except Exception as e:
        print(f"FAIL: invalid JSON from aiosh-mcp: {e}")
        print(stdout)
        sys.exit(1)


def call_mcp_tool(tool_name: str, arguments: dict | None = None, timeout_s: int = 30) -> dict:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments or {}},
    }
    resp = run_mcp(payload, timeout_s=timeout_s)
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


def list_mcp_tools(timeout_s: int = 30) -> list[dict]:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {},
    }
    resp = run_mcp(payload, timeout_s=timeout_s)
    return resp.get("result", {}).get("tools", [])


def test_tool_registration():
    print("TEST: tool registration via tools/list ...", end=" ")
    tools = list_mcp_tools()
    tool_names = {t.get("name") for t in tools}
    assert "aios.capability.observability" in tool_names, (
        f"aios.capability.observability not found in tools/list: {tool_names}"
    )
    print("OK")


def test_observability_lifecycle():
    print("TEST: capability observability report lifecycle over MCP ...", end=" ")
    with tempfile.TemporaryDirectory(prefix="aios_cap_obs_test_") as tmp_dir:
        store_path = os.path.join(tmp_dir, "capability_store.json")

        # 1. Report on empty registry
        res_empty = call_mcp_tool("aios.capability.observability", {"store_path": store_path})
        assert res_empty.get("ok") is True, f"Failed empty observability call: {res_empty}"
        report_empty = res_empty.get("report", {})
        assert report_empty.get("total_capabilities") == 0
        assert report_empty.get("is_healthy") is True

        # 2. Issue a root capability
        res_issue = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "agent:primary",
            "scope_type": "filesystem",
            "scope_target": "/workspace/data",
            "rights": ["read", "write", "delegate"],
            "max_invocations": 50,
            "store_path": store_path,
        })
        assert res_issue.get("ok") is True, f"Issue failed: {res_issue}"
        root_id = res_issue["capability"]["id"]

        # 3. Attenuate to a child capability
        res_att = call_mcp_tool("aios.capability.attenuate", {
            "parent_id": root_id,
            "new_subject": "agent:secondary",
            "subset_rights": ["read"],
            "store_path": store_path,
        })
        assert res_att.get("ok") is True, f"Attenuate failed: {res_att}"

        # 4. Observability report on populated registry
        res_pop = call_mcp_tool("aios.capability.observability", {"store_path": store_path})
        assert res_pop.get("ok") is True, f"Populated report failed: {res_pop}"
        report_pop = res_pop.get("report", {})
        assert report_pop.get("total_capabilities") == 2
        assert report_pop.get("root_capabilities") == 1
        assert report_pop.get("attenuated_capabilities") == 1
        assert report_pop.get("active_capabilities") == 2
        assert report_pop.get("max_derivation_depth") == 1
        assert report_pop.get("unique_subjects_count") == 2
        assert report_pop.get("is_healthy") is True

        # 5. Revoke root capability (cascade revokes child)
        res_revoke = call_mcp_tool("aios.capability.revoke", {
            "id": root_id,
            "store_path": store_path,
        })
        assert res_revoke.get("ok") is True, f"Revoke failed: {res_revoke}"

        # 6. Observability report after cascade revocation
        res_post_rev = call_mcp_tool("aios.capability.observability", {"store_path": store_path})
        assert res_post_rev.get("ok") is True, f"Post-revocation report failed: {res_post_rev}"
        report_post_rev = res_post_rev.get("report", {})
        assert report_post_rev.get("total_capabilities") == 2
        assert report_post_rev.get("active_capabilities") == 0
        assert report_post_rev.get("revoked_capabilities") == 2

    print("OK")


def main():
    print(f"Running Capability Observability Integration Smoke against binary: {get_mcp_binary()}")
    test_tool_registration()
    test_observability_lifecycle()
    print("ALL OBSERVABILITY INTEGRATION TESTS PASSED")


if __name__ == "__main__":
    main()
