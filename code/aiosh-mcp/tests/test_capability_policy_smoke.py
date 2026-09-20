#!/usr/bin/env python3
"""Integration Smoke Test for Capability Security Policy via MCP (T-02066).

Validates end-to-end policy enforcement on the MCP JSON-RPC boundary:
1. Prohibited filesystem path enforcement (/etc, /proc, /sys).
2. Prohibited network host enforcement (metadata IP/host).
3. Disallowed rights enforcement on untrusted subjects.
4. Policy enforcement during attenuation via MCP.
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


def test_prohibited_paths_over_mcp():
    print("TEST: prohibited filesystem path blocking over MCP ...", end=" ")
    with tempfile.TemporaryDirectory(prefix="aios_cap_policy_path_") as tmp_dir:
        store_path = os.path.join(tmp_dir, "capability_store.json")

        # 1. Prohibited path: /etc/shadow
        res_etc = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "agent:tester",
            "scope_type": "filesystem",
            "scope_target": "/etc/shadow",
            "rights": ["read"],
            "store_path": store_path,
        })
        assert res_etc.get("ok") is False, f"Expected /etc/shadow to be blocked: {res_etc}"
        err_msg = str(res_etc.get("error", "") or res_etc.get("text", ""))
        assert "CAPSEC_PROHIBITED_PATH" in err_msg, f"Expected CAPSEC_PROHIBITED_PATH in error: {err_msg}"

        # 2. Prohibited path: /proc/cpuinfo
        res_proc = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "agent:tester",
            "scope_type": "filesystem",
            "scope_target": "/proc/cpuinfo",
            "rights": ["read"],
            "store_path": store_path,
        })
        assert res_proc.get("ok") is False, f"Expected /proc/cpuinfo to be blocked: {res_proc}"

        # 3. Allowed path: /workspace/safe
        res_safe = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "agent:tester",
            "scope_type": "filesystem",
            "scope_target": "/workspace/safe",
            "rights": ["read", "write"],
            "store_path": store_path,
        })
        assert res_safe.get("ok") is True, f"Expected /workspace/safe to succeed: {res_safe}"

    print("OK")


def test_prohibited_network_hosts_over_mcp():
    print("TEST: prohibited network host blocking over MCP ...", end=" ")
    with tempfile.TemporaryDirectory(prefix="aios_cap_policy_net_") as tmp_dir:
        store_path = os.path.join(tmp_dir, "capability_store.json")

        # 1. Prohibited: 169.254.169.254
        res_meta = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "agent:tester",
            "scope_type": "network",
            "scope_target": "169.254.169.254",
            "rights": ["read"],
            "store_path": store_path,
        })
        assert res_meta.get("ok") is False, f"Expected 169.254.169.254 to be blocked: {res_meta}"
        err_msg = str(res_meta.get("error", "") or res_meta.get("text", ""))
        assert "CAPSEC_PROHIBITED_HOST" in err_msg, f"Expected CAPSEC_PROHIBITED_HOST in error: {err_msg}"

        # 2. Allowed: api.internal
        res_internal = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "agent:tester",
            "scope_type": "network",
            "scope_target": "api.internal",
            "rights": ["read"],
            "store_path": store_path,
        })
        assert res_internal.get("ok") is True, f"Expected api.internal to succeed: {res_internal}"

    print("OK")


def test_disallowed_rights_and_attenuation_over_mcp():
    print("TEST: disallowed rights & attenuation policy over MCP ...", end=" ")
    with tempfile.TemporaryDirectory(prefix="aios_cap_policy_rights_") as tmp_dir:
        store_path = os.path.join(tmp_dir, "capability_store.json")

        # 1. Untrusted subject requesting Admin blocked
        res_untrusted_admin = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "untrusted:worker_1",
            "scope_type": "filesystem",
            "scope_target": "/workspace/data",
            "rights": ["read", "admin"],
            "store_path": store_path,
        })
        assert res_untrusted_admin.get("ok") is False, f"Expected untrusted admin to be blocked: {res_untrusted_admin}"
        err_msg = str(res_untrusted_admin.get("error", "") or res_untrusted_admin.get("text", ""))
        assert "CAPSEC_DISALLOWED_RIGHT" in err_msg, f"Expected CAPSEC_DISALLOWED_RIGHT in error: {err_msg}"

        # 2. Issue valid root capability to trusted agent
        res_root = call_mcp_tool("aios.capability.issue", {
            "issuer": "kernel",
            "subject": "agent:trusted",
            "scope_type": "filesystem",
            "scope_target": "/workspace/data",
            "rights": ["read", "write", "delegate"],
            "store_path": store_path,
        })
        assert res_root.get("ok") is True, f"Expected root issuance to succeed: {res_root}"
        root_id = res_root["capability"]["id"]

        # 3. Attenuation to untrusted subject granting Delegate blocked
        res_att_del = call_mcp_tool("aios.capability.attenuate", {
            "parent_id": root_id,
            "new_subject": "untrusted:subworker",
            "subset_rights": ["delegate"],
            "store_path": store_path,
        })
        assert res_att_del.get("ok") is False, f"Expected untrusted delegate attenuation to be blocked: {res_att_del}"
        err_msg = str(res_att_del.get("error", "") or res_att_del.get("text", ""))
        assert "CAPSEC_DISALLOWED_RIGHT" in err_msg, f"Expected CAPSEC_DISALLOWED_RIGHT in error: {err_msg}"

        # 4. Attenuation to untrusted subject granting Read succeeds
        res_att_read = call_mcp_tool("aios.capability.attenuate", {
            "parent_id": root_id,
            "new_subject": "untrusted:subworker",
            "subset_rights": ["read"],
            "store_path": store_path,
        })
        assert res_att_read.get("ok") is True, f"Expected untrusted read attenuation to succeed: {res_att_read}"
        child_cap = res_att_read["capability"]
        assert child_cap["rights"] == ["read"]
        assert child_cap["subject"] == "untrusted:subworker"

    print("OK")


def main():
    print(f"Running Capability Policy Integration Smoke against binary: {get_mcp_binary()}")
    test_prohibited_paths_over_mcp()
    test_prohibited_network_hosts_over_mcp()
    test_disallowed_rights_and_attenuation_over_mcp()
    print("ALL POLICY INTEGRATION TESTS PASSED")


if __name__ == "__main__":
    main()
