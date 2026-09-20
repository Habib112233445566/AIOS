#!/usr/bin/env python3
"""Automated Cross-Surface Integration Smoke Test for Capability Model (Sub-Epic 6, T-02056).

Tests end-to-end multi-tier capability attenuation, cascade revocation, quota exhaustion,
and fault injection against the compiled aiosh-mcp binary over JSON-RPC.

Run standalone:
    python code/aiosh-mcp/tests/test_capability_automated_smoke.py
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


def run_mcp(payload: dict, env_overrides: dict[str, str] | None = None, timeout_s: int = 30) -> dict:
    env = os.environ.copy()
    if env_overrides:
        env.update(env_overrides)
    p = subprocess.Popen(
        [get_mcp_binary()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        env=env,
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


def call_mcp_tool(
    tool_name: str,
    arguments: dict | None = None,
    env_overrides: dict[str, str] | None = None,
    timeout_s: int = 30,
) -> dict:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments or {}},
    }
    resp = run_mcp(payload, env_overrides=env_overrides, timeout_s=timeout_s)
    if "error" in resp:
        return {"ok": False, "error": resp["error"]}
    res = resp.get("result", {})
    if "structuredContent" in res and "result" in res["structuredContent"]:
        data = res["structuredContent"]["result"]
    elif "content" in res and res["content"]:
        try:
            data = json.loads(res["content"][0]["text"])
        except Exception:
            data = {"ok": False, "text": res["content"][0]["text"]}
    else:
        data = res
    return data


def test_multitier_lifecycle_and_cascade_revocation():
    print("TEST: Multi-tier capability attenuation and cascade revocation ... ", end="", flush=True)
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "automated_store.json")

        # 1. Issue Root Capability (Tier 0)
        res_t0 = call_mcp_tool(
            "aios.capability.issue",
            {
                "issuer": "kernel",
                "subject": "agent:tier0",
                "scope_type": "filesystem",
                "scope_target": "/var",
                "rights": ["read", "write", "delegate"],
                "store_path": store_path,
            },
        )
        assert res_t0.get("ok") is True, f"Tier 0 issue failed: {res_t0}"
        id_t0 = res_t0["capability"]["id"]

        # 2. Attenuate to Tier 1
        res_t1 = call_mcp_tool(
            "aios.capability.attenuate",
            {
                "parent_id": id_t0,
                "new_subject": "agent:tier1",
                "scope_type": "filesystem",
                "scope_target": "/var/data",
                "subset_rights": ["read", "write", "delegate"],
                "store_path": store_path,
            },
        )
        assert res_t1.get("ok") is True, f"Tier 1 attenuate failed: {res_t1}"
        id_t1 = res_t1["capability"]["id"]

        # 3. Attenuate to Tier 2
        res_t2 = call_mcp_tool(
            "aios.capability.attenuate",
            {
                "parent_id": id_t1,
                "new_subject": "agent:tier2",
                "scope_type": "filesystem",
                "scope_target": "/var/data/sub",
                "subset_rights": ["read", "delegate"],
                "store_path": store_path,
            },
        )
        assert res_t2.get("ok") is True, f"Tier 2 attenuate failed: {res_t2}"
        id_t2 = res_t2["capability"]["id"]

        # 4. Attenuate to Tier 3
        res_t3 = call_mcp_tool(
            "aios.capability.attenuate",
            {
                "parent_id": id_t2,
                "new_subject": "agent:tier3",
                "subset_rights": ["read"],
                "store_path": store_path,
            },
        )
        assert res_t3.get("ok") is True, f"Tier 3 attenuate failed: {res_t3}"
        id_t3 = res_t3["capability"]["id"]

        # 5. Check rights before revocation
        check_t3_read = call_mcp_tool(
            "aios.capability.check",
            {
                "subject": "agent:tier3",
                "scope_type": "filesystem",
                "scope_target": "/var/data/sub",
                "right": "read",
                "store_path": store_path,
            },
        )
        assert check_t3_read.get("granted") is True

        check_t3_write = call_mcp_tool(
            "aios.capability.check",
            {
                "subject": "agent:tier3",
                "scope_type": "filesystem",
                "scope_target": "/var/data/sub",
                "right": "write",
                "store_path": store_path,
            },
        )
        assert check_t3_write.get("granted") is False

        # 6. Revoke at Tier 1 -> Must cascade to Tier 1, Tier 2, Tier 3
        revoke_res = call_mcp_tool(
            "aios.capability.revoke",
            {"id": id_t1, "store_path": store_path},
        )
        assert revoke_res.get("ok") is True
        revoked_ids = revoke_res.get("revoked_ids", [])
        assert id_t1 in revoked_ids
        assert id_t2 in revoked_ids
        assert id_t3 in revoked_ids

        # 7. Verify Tier 3 access denied post-revocation
        check_t3_post = call_mcp_tool(
            "aios.capability.check",
            {
                "subject": "agent:tier3",
                "scope_type": "filesystem",
                "scope_target": "/var/data/sub",
                "right": "read",
                "store_path": store_path,
            },
        )
        assert check_t3_post.get("granted") is False

        # 8. Verify Root (Tier 0) remains active
        check_t0_post = call_mcp_tool(
            "aios.capability.check",
            {
                "subject": "agent:tier0",
                "scope_type": "filesystem",
                "scope_target": "/var",
                "right": "read",
                "store_path": store_path,
            },
        )
        assert check_t0_post.get("granted") is True
    print("OK")


def test_quota_exhaustion():
    print("TEST: Invocation quota exhaustion over MCP ... ", end="", flush=True)
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "quota_store.json")

        issue_res = call_mcp_tool(
            "aios.capability.issue",
            {
                "issuer": "kernel",
                "subject": "agent:metered",
                "scope_type": "filesystem",
                "scope_target": "/data",
                "rights": ["read"],
                "max_invocations": 2,
                "store_path": store_path,
            },
        )
        assert issue_res.get("ok") is True

        # Invocations 1 and 2
        for _ in range(2):
            res = call_mcp_tool(
                "aios.capability.check",
                {
                    "subject": "agent:metered",
                    "scope_type": "filesystem",
                    "scope_target": "/data",
                    "right": "read",
                    "consume": True,
                    "store_path": store_path,
                },
            )
            assert res.get("granted") is True

        # Invocation 3 should be denied
        res_denied = call_mcp_tool(
            "aios.capability.check",
            {
                "subject": "agent:metered",
                "scope_type": "filesystem",
                "scope_target": "/data",
                "right": "read",
                "consume": True,
                "store_path": store_path,
            },
        )
        assert res_denied.get("granted") is False
    print("OK")


def test_fault_injection():
    print("TEST: Fault injection and path protection ... ", end="", flush=True)
    # Path traversal in store_path
    res_trav = call_mcp_tool(
        "aios.capability.get",
        {"id": "cap_test", "store_path": "../evil.json"},
    )
    assert res_trav.get("ok") is False or "error" in res_trav

    # Non-json extension
    res_ext = call_mcp_tool(
        "aios.capability.list",
        {"store_path": "store.txt"},
    )
    assert res_ext.get("ok") is False or "error" in res_ext
    print("OK")


def main() -> int:
    print(f"Running Capability Automated Smoke against binary: {get_mcp_binary()}")
    test_multitier_lifecycle_and_cascade_revocation()
    test_quota_exhaustion()
    test_fault_injection()
    print("ALL AUTOMATED TESTS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
