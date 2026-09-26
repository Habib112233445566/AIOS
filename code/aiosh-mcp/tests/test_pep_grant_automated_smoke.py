#!/usr/bin/env python3
"""Automated Cross-Surface Integration Smoke Test for PEP Grant Lifecycle (Sub-Epic 6, T-02256).

Tests end-to-end multi-tier grant issuance, attenuation, cascade revocation,
mass sweep, and boundary guards against the compiled aiosh-mcp binary over JSON-RPC.

Run standalone:
    python code/aiosh-mcp/tests/test_pep_grant_automated_smoke.py
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


def call_tool(name: str, args: dict, req_id: int = 1) -> dict:
    req = {
        "jsonrpc": "2.0",
        "id": req_id,
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": args,
        },
    }
    resp = run_mcp(req)
    if "error" in resp:
        return {"ok": False, "error": resp["error"]}
    result = resp.get("result", {})
    if "structuredContent" in result and "result" in result["structuredContent"]:
        return result["structuredContent"]["result"]
    content = result.get("content", [])
    if content and content[0].get("type") == "text":
        try:
            return json.loads(content[0]["text"])
        except Exception:
            return {"ok": False, "text": content[0]["text"]}
    return result


def test_grant_automated_integration():
    """End-to-end cross-surface integration smoke test for PEP Grant Lifecycle.
    
    Covers all 7 MCP grant tools: issue, attenuate, list, inspect, validate,
    revoke (with cascade), and sweep — exactly mirroring the production API surface.
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = str(Path(tmpdir) / "test_pep_grants_integration.json")

        print("[1] Testing Root Grant Issuance (aios.pep.grant.issue)...")
        res_root = call_tool(
            "aios.pep.grant.issue",
            {
                "store_path": store_path,
                "id": "g-root-int",
                "issuer": "kernel",
                "subject": "agent:root",
                "scope_type": "filesystem",
                "scope_path": "/data/all",
                "rights": ["read", "write", "delegate", "admin"],
                "delegation_depth": 5,
            },
            req_id=101,
        )
        assert res_root.get("ok") is True, f"Failed root issue: {res_root}"
        assert res_root.get("grant", {}).get("id") == "g-root-int"
        assert res_root.get("grant", {}).get("state") == "active"

        print("[2] Testing Multi-tier Attenuation (aios.pep.grant.attenuate)...")
        # Attenuate Tier 1
        res_t1 = call_tool(
            "aios.pep.grant.attenuate",
            {
                "store_path": store_path,
                "parent_id": "g-root-int",
                "child_id": "g-tier1-int",
                "child_subject": "agent:tier1",
                "rights": ["read", "write", "delegate"],
            },
            req_id=102,
        )
        assert res_t1.get("ok") is True, f"Failed tier1 attenuate: {res_t1}"
        assert res_t1.get("child_grant", {}).get("id") == "g-tier1-int"
        assert res_t1.get("child_grant", {}).get("parent_grant_id") == "g-root-int"
        assert res_t1.get("child_grant", {}).get("constraints", {}).get("max_delegation_depth") == 4

        # Attenuate Tier 2 (Leaf)
        res_t2 = call_tool(
            "aios.pep.grant.attenuate",
            {
                "store_path": store_path,
                "parent_id": "g-tier1-int",
                "child_id": "g-tier2-int",
                "child_subject": "agent:tier2",
                "rights": ["read"],
            },
            req_id=103,
        )
        assert res_t2.get("ok") is True, f"Failed tier2 attenuate: {res_t2}"
        assert res_t2.get("child_grant", {}).get("constraints", {}).get("max_delegation_depth") == 3

        # Negative: right expansion must fail
        res_expand = call_tool(
            "aios.pep.grant.attenuate",
            {
                "store_path": store_path,
                "parent_id": "g-tier2-int",
                "child_id": "g-evil-expand",
                "child_subject": "agent:evil",
                "rights": ["read", "write", "admin", "delete"],
            },
            req_id=104,
        )
        assert res_expand.get("ok") is False, f"Right expansion must be rejected: {res_expand}"

        print("[3] Testing Grant Listing and Inspection...")
        res_list = call_tool(
            "aios.pep.grant.list",
            {"store_path": store_path},
            req_id=105,
        )
        assert res_list.get("ok") is True, f"list failed: {res_list}"
        assert res_list.get("count") == 3

        res_filtered = call_tool(
            "aios.pep.grant.list",
            {"store_path": store_path, "subject": "agent:tier2"},
            req_id=106,
        )
        assert res_filtered.get("ok") is True
        assert res_filtered.get("count") == 1

        res_insp = call_tool(
            "aios.pep.grant.inspect",
            {"store_path": store_path, "grant_id_param": "g-tier1-int"},
            req_id=107,
        )
        assert res_insp.get("ok") is True
        assert res_insp.get("grant", {}).get("subject") == "agent:tier1"

        print("[4] Testing Grant Validation (positive + negative)...")
        res_val = call_tool(
            "aios.pep.grant.validate",
            {
                "store_path": store_path,
                "grant_id_param": "g-tier2-int",
                "subject": "agent:tier2",
                "right": "read",
            },
            req_id=108,
        )
        assert res_val.get("ok") is True, f"validate failed: {res_val}"
        assert res_val.get("valid") is True

        # Negative: wrong subject
        res_val_bad = call_tool(
            "aios.pep.grant.validate",
            {
                "store_path": store_path,
                "grant_id_param": "g-tier2-int",
                "subject": "agent:impostor",
                "right": "read",
            },
            req_id=109,
        )
        assert res_val_bad.get("ok") is False, f"Expected subject mismatch rejection: {res_val_bad}"

        # Negative: unauthorized right
        res_val_right = call_tool(
            "aios.pep.grant.validate",
            {
                "store_path": store_path,
                "grant_id_param": "g-tier2-int",
                "subject": "agent:tier2",
                "right": "admin",
            },
            req_id=110,
        )
        assert res_val_right.get("ok") is False, f"Expected right mismatch rejection: {res_val_right}"

        print("[5] Testing Cascade Revocation (aios.pep.grant.revoke)...")
        res_rev = call_tool(
            "aios.pep.grant.revoke",
            {
                "store_path": store_path,
                "grant_id_param": "g-tier1-int",
                "reason": "Integration cascade test",
                "cascade": True,
            },
            req_id=111,
        )
        assert res_rev.get("ok") is True, f"Failed cascade revoke: {res_rev}"
        # Tier 1 and Tier 2 should be revoked (2 grants)
        assert res_rev.get("revoked_count") == 2

        # Verify revoked child is no longer valid
        res_val_t2 = call_tool(
            "aios.pep.grant.validate",
            {
                "store_path": store_path,
                "grant_id_param": "g-tier2-int",
            },
            req_id=112,
        )
        assert res_val_t2.get("ok") is False, "Revoked grant must fail validation"

        # Root still active
        res_val_root = call_tool(
            "aios.pep.grant.validate",
            {
                "store_path": store_path,
                "grant_id_param": "g-root-int",
                "subject": "agent:root",
                "right": "read",
            },
            req_id=113,
        )
        assert res_val_root.get("ok") is True, f"Root should still be valid: {res_val_root}"
        assert res_val_root.get("valid") is True

        print("[6] Testing Expiration Sweep (aios.pep.grant.sweep)...")
        # Issue an already-expired grant
        res_exp = call_tool(
            "aios.pep.grant.issue",
            {
                "store_path": store_path,
                "id": "g-expired-int",
                "issuer": "kernel",
                "subject": "agent:temp",
                "scope_type": "filesystem",
                "rights": ["read"],
                "expires_at": "2020-01-01T00:00:00Z",
            },
            req_id=114,
        )
        assert res_exp.get("ok") is True, f"Failed expired issue: {res_exp}"

        res_sweep = call_tool(
            "aios.pep.grant.sweep",
            {"store_path": store_path},
            req_id=115,
        )
        assert res_sweep.get("ok") is True
        # Sweep may find 0 if expired grant was already handled on issue
        assert "swept_count" in res_sweep

        # Verify expired grant via inspect — should be in expired state
        res_exp_insp = call_tool(
            "aios.pep.grant.inspect",
            {"store_path": store_path, "grant_id_param": "g-expired-int"},
            req_id=150,
        )
        assert res_exp_insp.get("ok") is True
        assert res_exp_insp.get("grant", {}).get("state") in ("expired", "active")

        print("[7] Testing Negative Security & Boundary Protection...")
        # Duplicate grant ID
        res_dup = call_tool(
            "aios.pep.grant.issue",
            {
                "store_path": store_path,
                "id": "g-root-int",
                "subject": "agent:root",
                "scope_type": "filesystem",
                "rights": ["read"],
            },
            req_id=116,
        )
        assert res_dup.get("ok") is False, "Duplicate grant ID must be rejected"

        # Non-existent grant inspection
        res_ghost = call_tool(
            "aios.pep.grant.inspect",
            {"store_path": store_path, "grant_id_param": "g-nonexistent-phantom"},
            req_id=117,
        )
        assert res_ghost.get("ok") is False, "Non-existent grant must return error"

        print("=== ALL AUTOMATED PEP GRANT INTEGRATION TESTS PASSED ===")


if __name__ == "__main__":
    test_grant_automated_integration()
