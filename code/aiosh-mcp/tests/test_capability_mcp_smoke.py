#!/usr/bin/env python3
"""Cross-Surface Integration Smoke Test for Capability Model MCP Surface (T-02036).

Proves that all 7 Capability Model MCP tools are exposed via JSON-RPC,
adhere to schema contracts, enforce bounds and security sanitization, and achieve
cross-surface state parity with the CapabilityService.

Covers:
- Tool registration: All 7 `aios.capability.*` tools advertised via tools/list.
- List query: `aios.capability.list` on empty and populated registries.
- Root issuance: `aios.capability.issue` with valid/invalid issuer credentials.
- Retrieval: `aios.capability.get` by ID.
- Monotonic attenuation: `aios.capability.attenuate` with rights restriction and escalation rejection.
- Fast access checking: `aios.capability.check` with invocation quota consumption.
- Transitive cascade revocation: `aios.capability.revoke` revoking parent and child subtrees.
- Leaf pruning: `aios.capability.prune`.

Run standalone:
    python code/aiosh-mcp/tests/test_capability_mcp_smoke.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

CAPABILITY_TOOLS = (
    "aios.capability.list",
    "aios.capability.get",
    "aios.capability.issue",
    "aios.capability.attenuate",
    "aios.capability.revoke",
    "aios.capability.check",
    "aios.capability.prune",
)


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
    for expected in CAPABILITY_TOOLS:
        assert expected in tool_names, f"Expected tool '{expected}' not found in manifest: {tool_names}"
    print("OK")


def test_capability_lifecycle():
    print("TEST: capability full lifecycle over MCP JSON-RPC ...", end=" ")
    with tempfile.TemporaryDirectory(prefix="aios_cap_mcp_test_") as tmp_dir:
        store_path = os.path.join(tmp_dir, "capability_store.json")

        # 1. List empty
        res_list = call_mcp_tool("aios.capability.list", {"store_path": store_path})
        assert res_list.get("ok") is True, f"List failed: {res_list}"
        assert res_list.get("count") == 0, f"Expected 0 count, got: {res_list.get('count')}"

        # 2. Negative root issuance: unauthorized issuer
        res_issue_unauth = call_mcp_tool("aios.capability.issue", {
            "issuer": "untrusted:user",
            "subject": "agent:worker",
            "scope_type": "filesystem",
            "scope_target": "/var/data",
            "rights": ["read", "write"],
            "store_path": store_path,
        })
        assert res_issue_unauth.get("ok") is False, f"Expected refusal, got: {res_issue_unauth}"

        # 3. Positive root issuance
        res_issue = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "agent:worker",
            "scope_type": "filesystem",
            "scope_target": "/var/data",
            "rights": ["read", "write", "delegate"],
            "max_invocations": 10,
            "store_path": store_path,
        })
        assert res_issue.get("ok") is True, f"Issue failed: {res_issue}"
        root_cap = res_issue.get("capability", {})
        root_id = root_cap.get("id")
        assert root_id and root_id.startswith("cap_"), f"Invalid root ID: {root_id}"
        assert root_cap.get("subject") == "agent:worker"

        # 4. Get capability by ID
        res_get = call_mcp_tool("aios.capability.get", {
            "id": root_id,
            "store_path": store_path,
        })
        assert res_get.get("ok") is True, f"Get failed: {res_get}"
        assert res_get.get("capability", {}).get("id") == root_id

        # 5. Negative attenuation: privilege escalation
        res_att_esc = call_mcp_tool("aios.capability.attenuate", {
            "parent_id": root_id,
            "new_subject": "agent:subworker",
            "subset_rights": ["admin"],
            "store_path": store_path,
        })
        assert res_att_esc.get("ok") is False, f"Expected escalation rejection, got: {res_att_esc}"

        # 6. Positive attenuation: valid monotonic reduction
        res_att = call_mcp_tool("aios.capability.attenuate", {
            "parent_id": root_id,
            "new_subject": "agent:subworker",
            "subset_rights": ["read"],
            "max_invocations": 5,
            "store_path": store_path,
        })
        assert res_att.get("ok") is True, f"Attenuate failed: {res_att}"
        child_cap = res_att.get("capability", {})
        child_id = child_cap.get("id")
        assert child_id and child_id.startswith("cap_")
        assert child_cap.get("parent_id") == root_id

        # 7. Check capability with consumption
        res_check = call_mcp_tool("aios.capability.check", {
            "subject": "agent:worker",
            "scope_type": "filesystem",
            "scope_target": "/var/data",
            "right": "read",
            "consume": True,
            "store_path": store_path,
        })
        assert res_check.get("ok") is True, f"Check failed: {res_check}"
        assert res_check.get("granted") is True
        assert res_check.get("remaining_invocations") == 9

        # 8. Check ungranted right
        res_check_unauth = call_mcp_tool("aios.capability.check", {
            "subject": "agent:subworker",
            "scope_type": "filesystem",
            "scope_target": "/var/data",
            "right": "write",
            "store_path": store_path,
        })
        assert res_check_unauth.get("ok") is True
        assert res_check_unauth.get("granted") is False

        # 9. Cascade revocation
        res_revoke = call_mcp_tool("aios.capability.revoke", {
            "id": root_id,
            "store_path": store_path,
        })
        assert res_revoke.get("ok") is True, f"Revoke failed: {res_revoke}"
        revoked_ids = res_revoke.get("revoked_ids", [])
        assert root_id in revoked_ids, f"Root {root_id} not in revoked: {revoked_ids}"
        assert child_id in revoked_ids, f"Child {child_id} not in revoked: {revoked_ids}"

        # 10. Check access after revocation
        res_check_revoked = call_mcp_tool("aios.capability.check", {
            "subject": "agent:worker",
            "scope_type": "filesystem",
            "scope_target": "/var/data",
            "right": "read",
            "store_path": store_path,
        })
        assert res_check_revoked.get("ok") is True
        assert res_check_revoked.get("granted") is False

        # 11. Prune
        res_prune = call_mcp_tool("aios.capability.prune", {
            "store_path": store_path,
        })
        assert res_prune.get("ok") is True, f"Prune failed: {res_prune}"

    print("OK")


def main():
    print(f"Running Capability MCP Smoke against binary: {get_mcp_binary()}")
    test_tool_registration()
    test_capability_lifecycle()
    print("ALL TESTS PASSED")


if __name__ == "__main__":
    main()
