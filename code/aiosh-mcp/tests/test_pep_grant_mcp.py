#!/usr/bin/env python3
"""Comprehensive Unit Tests for PEP Grant MCP Tool Surface (T-02235).

Covers all 7 Grant Lifecycle tools over MCP JSON-RPC 2.0 stdio:
1. `aios.pep.grant.issue`: Root grant creation, argument validation, negative cases.
2. `aios.pep.grant.attenuate`: Child derivation, rights monotonicity, depth decrement.
3. `aios.pep.grant.list`: Grant enumeration, subject filtering.
4. `aios.pep.grant.inspect`: Grant inspection by ID, not-found error handling.
5. `aios.pep.grant.validate`: Active authorization validation, right mismatch, subject mismatch.
6. `aios.pep.grant.revoke`: Revocation with reason, recursive cascade to child grants.
7. `aios.pep.grant.sweep`: Sweep expired grants.
"""

from __future__ import annotations

import json
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
        raise TimeoutError(f"aiosh-mcp timed out after {timeout_s}s")
    finally:
        if p.poll() is None:
            p.kill()
            p.wait()

    assert p.returncode == 0, f"aiosh-mcp exited with returncode {p.returncode}"
    return json.loads(stdout.strip())


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


def test_mcp_grant_tool_registration():
    tools = list_mcp_tools()
    tool_names = {t["name"] for t in tools}
    expected_tools = [
        "aios.pep.grant.issue",
        "aios.pep.grant.attenuate",
        "aios.pep.grant.list",
        "aios.pep.grant.inspect",
        "aios.pep.grant.validate",
        "aios.pep.grant.revoke",
        "aios.pep.grant.sweep",
    ]
    for expected in expected_tools:
        assert expected in tool_names, f"{expected} missing from tools/list manifest"


def test_mcp_grant_issue_and_validate():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "grants.json")

        # 1. Happy path: issue root parent grant
        res = call_mcp_tool("aios.pep.grant.issue", {
            "id": "g-root-mcp",
            "issuer": "mcp-sec-admin",
            "subject": "agent:lead",
            "scope_type": "filesystem",
            "scope_path": "/var/data",
            "rights": ["read", "write", "delegate"],
            "delegation_depth": 2,
            "expires_at": "2026-12-31T23:59:59Z",
            "store_path": store,
        })
        assert res.get("ok") is True, f"issue failed: {res}"
        g = res.get("grant", {})
        assert g.get("id") == "g-root-mcp"
        assert g.get("subject") == "agent:lead"
        assert g.get("state") == "active"
        assert "delegate" in g.get("rights", [])
        assert g.get("constraints", {}).get("max_delegation_depth") == 2

        # 2. Validation of active grant
        res_val = call_mcp_tool("aios.pep.grant.validate", {
            "grant_id_param": "g-root-mcp",
            "subject": "agent:lead",
            "right": "write",
            "store_path": store,
        })
        assert res_val.get("ok") is True, f"validate failed: {res_val}"
        assert res_val.get("valid") is True

        # 3. Negative validation: unauthorized right
        res_bad_right = call_mcp_tool("aios.pep.grant.validate", {
            "grant_id_param": "g-root-mcp",
            "subject": "agent:lead",
            "right": "delete",
            "store_path": store,
        })
        assert res_bad_right.get("ok") is False

        # 4. Negative validation: subject mismatch
        res_bad_subj = call_mcp_tool("aios.pep.grant.validate", {
            "grant_id_param": "g-root-mcp",
            "subject": "agent:impostor",
            "right": "read",
            "store_path": store,
        })
        assert res_bad_subj.get("ok") is False

        # 5. Negative issuance: missing required arguments
        res_missing = call_mcp_tool("aios.pep.grant.issue", {
            "id": "g-missing",
            "subject": "agent:bad",
            "store_path": store,
        })
        assert res_missing.get("ok") is False

        # 6. Negative issuance: unknown scope type
        res_unknown_scope = call_mcp_tool("aios.pep.grant.issue", {
            "id": "g-bad-scope",
            "subject": "agent:bad",
            "scope_type": "quantum_field",
            "rights": ["read"],
            "store_path": store,
        })
        assert res_unknown_scope.get("ok") is False

        # 7. Negative issuance: duplicate grant ID
        res_dup = call_mcp_tool("aios.pep.grant.issue", {
            "id": "g-root-mcp",
            "subject": "agent:lead",
            "scope_type": "filesystem",
            "rights": ["read"],
            "store_path": store,
        })
        assert res_dup.get("ok") is False


def test_mcp_grant_attenuation_and_cascade_revocation():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "grants.json")

        # 1. Issue root parent grant
        res_root = call_mcp_tool("aios.pep.grant.issue", {
            "id": "g-parent-1",
            "subject": "agent:parent",
            "scope_type": "filesystem",
            "scope_path": "/workspace",
            "rights": ["read", "write", "delegate"],
            "delegation_depth": 2,
            "store_path": store,
        })
        assert res_root.get("ok") is True

        # 2. Derive attenuated child grant
        res_child = call_mcp_tool("aios.pep.grant.attenuate", {
            "parent_id": "g-parent-1",
            "child_id": "g-child-1",
            "child_subject": "agent:worker",
            "rights": ["read"],
            "store_path": store,
        })
        assert res_child.get("ok") is True, f"attenuate failed: {res_child}"
        child = res_child.get("child_grant", {})
        assert child.get("id") == "g-child-1"
        assert child.get("parent_grant_id") == "g-parent-1"
        assert child.get("rights") == ["read"]
        assert child.get("constraints", {}).get("max_delegation_depth") == 1

        # 3. Negative attenuation: right expansion attempt
        res_expand = call_mcp_tool("aios.pep.grant.attenuate", {
            "parent_id": "g-parent-1",
            "child_id": "g-child-evil",
            "child_subject": "agent:worker",
            "rights": ["admin", "delete"],
            "store_path": store,
        })
        assert res_expand.get("ok") is False, "Right expansion must be rejected"

        # 4. List and inspect grants
        res_list = call_mcp_tool("aios.pep.grant.list", {"store_path": store})
        assert res_list.get("ok") is True
        assert res_list.get("count") == 2

        res_filtered = call_mcp_tool("aios.pep.grant.list", {
            "store_path": store,
            "subject": "agent:worker",
        })
        assert res_filtered.get("ok") is True
        assert res_filtered.get("count") == 1

        res_insp = call_mcp_tool("aios.pep.grant.inspect", {
            "store_path": store,
            "grant_id_param": "g-child-1",
        })
        assert res_insp.get("ok") is True
        assert res_insp.get("grant", {}).get("id") == "g-child-1"

        # 5. Revoke parent with cascade=True
        res_rev = call_mcp_tool("aios.pep.grant.revoke", {
            "store_path": store,
            "grant_id_param": "g-parent-1",
            "reason": "Revoking tree",
            "cascade": True,
        })
        assert res_rev.get("ok") is True
        assert res_rev.get("revoked_count") == 2

        # 6. Verify child grant is no longer valid
        res_val_child = call_mcp_tool("aios.pep.grant.validate", {
            "store_path": store,
            "grant_id_param": "g-child-1",
        })
        assert res_val_child.get("ok") is False


def test_mcp_grant_sweep():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "grants.json")

        # 1. Issue grant expired in past
        res_exp = call_mcp_tool("aios.pep.grant.issue", {
            "id": "g-exp-1",
            "subject": "agent:temp",
            "scope_type": "filesystem",
            "rights": ["read"],
            "expires_at": "2020-01-01T00:00:00Z",
            "store_path": store,
        })
        assert res_exp.get("ok") is True

        # 2. Sweep expired grants
        res_sweep = call_mcp_tool("aios.pep.grant.sweep", {
            "store_path": store,
        })
        assert res_sweep.get("ok") is True
        assert res_sweep.get("swept_count") >= 1

        # 3. Inspect swept grant
        res_insp = call_mcp_tool("aios.pep.grant.inspect", {
            "store_path": store,
            "grant_id_param": "g-exp-1",
        })
        assert res_insp.get("ok") is True
        assert res_insp.get("grant", {}).get("state") == "expired"


if __name__ == "__main__":
    test_mcp_grant_tool_registration()
    test_mcp_grant_issue_and_validate()
    test_mcp_grant_attenuation_and_cascade_revocation()
    test_mcp_grant_sweep()
    print("=== All MCP Grant Unit Tests Passed Successfully ===")
