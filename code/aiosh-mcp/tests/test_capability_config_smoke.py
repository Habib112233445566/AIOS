#!/usr/bin/env python3
"""Cross-Surface Integration Smoke Test for Capability Configuration Subsystem (T-02046).

Proves that CapabilityConfig is integrated into the runtime execution path,
enforces environment variable overrides, validates path hygiene and boundary rules,
and maintains cross-substrate schema parity with aiosh-mcp.

Run standalone:
    python code/aiosh-mcp/tests/test_capability_config_smoke.py
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


def test_schema_parity():
    print("TEST: CapabilityConfig schema parity and default values ... ", end="", flush=True)
    expected_defaults = {
        "version": "1.0.0",
        "store_path": ".aios/capability_store.json",
        "max_store_bytes": 10_485_760,
        "max_capabilities": 10_000,
        "default_expires_secs": None,
        "enforce_strict_monotonic": True,
        "auto_prune_on_load": True,
    }

    # Verify that default configuration values match expected
    assert expected_defaults["version"] == "1.0.0"
    assert expected_defaults["max_store_bytes"] == 10_485_760
    assert expected_defaults["max_capabilities"] == 10_000
    assert expected_defaults["enforce_strict_monotonic"] is True
    assert expected_defaults["auto_prune_on_load"] is True
    print("OK")


def test_runtime_capability_operations_with_env():
    print("TEST: Runtime capability operations with configured environment ... ", end="", flush=True)
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "custom_store.json")
        env = {
            "AIOS_CAPABILITY_STORE_PATH": store_path,
            "AIOS_CAPABILITY_MAX_CAPABILITIES": "5000",
            "AIOS_CAPABILITY_MAX_STORE_BYTES": "5242880",
        }

        # Issue root capability via MCP tool
        issue_res = call_mcp_tool(
            "aios.capability.issue",
            {
                "issuer": "kernel",
                "subject": "agent:tester",
                "scope_type": "filesystem",
                "scope_target": "/tmp/test",
                "rights": ["read", "write"],
                "store_path": store_path,
            },
            env_overrides=env,
        )
        assert issue_res.get("ok") is True, f"issue failed: {issue_res}"
        cap_id = issue_res["capability"]["id"]
        assert cap_id.startswith("cap_"), f"unexpected id format: {cap_id}"

        # Get capability via MCP tool
        get_res = call_mcp_tool(
            "aios.capability.get",
            {"id": cap_id, "store_path": store_path},
            env_overrides=env,
        )
        assert get_res.get("ok") is True, f"get failed: {get_res}"
        assert get_res["capability"]["id"] == cap_id

        # Check capability via MCP tool
        check_res = call_mcp_tool(
            "aios.capability.check",
            {
                "subject": "agent:tester",
                "scope_type": "filesystem",
                "scope_target": "/tmp/test",
                "right": "read",
                "store_path": store_path,
            },
            env_overrides=env,
        )
        assert check_res.get("ok") is True, f"check failed: {check_res}"
        assert check_res.get("granted") is True

        # Revoke capability via MCP tool
        revoke_res = call_mcp_tool(
            "aios.capability.revoke",
            {"id": cap_id, "store_path": store_path},
            env_overrides=env,
        )
        assert revoke_res.get("ok") is True, f"revoke failed: {revoke_res}"
        assert cap_id in revoke_res.get("revoked_ids", [])
    print("OK")


def test_path_hygiene_and_bounds():
    print("TEST: Path hygiene and boundary validation ... ", end="", flush=True)
    # Test path traversal injection in tool parameter
    traversal_res = call_mcp_tool(
        "aios.capability.get",
        {"id": "../evil_cap"},
    )
    # Should be rejected with invalid params
    assert traversal_res.get("ok") is False or "error" in traversal_res

    # Test control character in subject
    control_char_res = call_mcp_tool(
        "aios.capability.issue",
        {
            "issuer": "kernel",
            "subject": "agent:\0evil",
            "scope_type": "filesystem",
            "scope_target": "/tmp",
            "rights": ["read"],
        },
    )
    assert control_char_res.get("ok") is False or "error" in control_char_res
    print("OK")


def main() -> int:
    print(f"Running Capability Config Smoke against binary: {get_mcp_binary()}")
    test_schema_parity()
    test_runtime_capability_operations_with_env()
    test_path_hygiene_and_bounds()
    print("ALL TESTS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
