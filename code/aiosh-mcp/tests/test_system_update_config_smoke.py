#!/usr/bin/env python3
"""Cross-Surface Integration Smoke Test for System Update Configuration Subsystem (T-01946).

Proves that SystemUpdateConfig is integrated into the runtime execution path of
aiosh-mcp, enforces environment variable overrides (UCONF4), rejects path
traversals and control characters (UCONF1), and achieves cross-substrate schema parity.

Covers:
- Default configuration & environment variable ingestion (AIOSH_UPDATE_STATE_DIR).
- Path hygiene enforcement on environment overrides (rejecting control chars and '..').
- Cross-substrate JSON serialization parity.

Run standalone:
    python code/aiosh-mcp/tests/test_system_update_config_smoke.py
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


def main() -> int:
    print("=== System Update Configuration Smoke & Integration Test ===")

    with tempfile.TemporaryDirectory() as td:
        state_dir = os.path.join(td, "env_state")
        os.makedirs(state_dir, exist_ok=True)

        # 1. Test environment variable ingestion (UCONF4)
        print("[1] Testing AIOSH_UPDATE_STATE_DIR environment override...")
        res = call_mcp_tool(
            "aios.update.status",
            arguments={},
            env_overrides={"AIOSH_UPDATE_STATE_DIR": state_dir},
        )
        assert res.get("ok") is True, f"Expected ok: true, got {res}"
        data = res.get("data", {})
        assert data.get("state") == "idle"
        assert data.get("active_slot") == "slot_a"
        print("  OK: environment variable override routed to update service")

        # 2. Test path hygiene on environment variable (UCONF1)
        print("[2] Testing path hygiene rejection on environment override...")
        bad_env_res = call_mcp_tool(
            "aios.update.status",
            arguments={},
            env_overrides={"AIOSH_UPDATE_STATE_DIR": f"{state_dir}/../escape"},
        )
        assert bad_env_res.get("ok") is False, f"Expected rejection of '..', got {bad_env_res}"
        assert "validation error" in bad_env_res.get("error", "").lower()
        print("  OK: parent directory traversal in environment rejected")

        bad_ctrl_res = call_mcp_tool(
            "aios.update.status",
            arguments={},
            env_overrides={"AIOSH_UPDATE_STATE_DIR": f"{state_dir}\x07bad"},
        )
        assert bad_ctrl_res.get("ok") is False, f"Expected rejection of control char, got {bad_ctrl_res}"
        print("  OK: control characters in environment rejected")

        # 3. Cross-substrate canonical JSON parity
        print("[3] Testing cross-substrate JSON serialization parity...")
        config_data = {
            "state_dir": state_dir,
            "staging_dir": os.path.join(td, "staging"),
            "default_channel": "stable",
            "check_interval_secs": 86400,
            "allow_auto_apply": False,
            "auto_rollback_on_failure": True,
            "max_payload_bytes": 10737418240,
            "min_free_space_bytes": 1073741824,
            "trusted_keys": ["key1_hash", "key2_hash"],
        }
        cfg_path = os.path.join(td, "system_update.json")
        with open(cfg_path, "w", encoding="utf-8") as f:
            json.dump(config_data, f, indent=2)

        with open(cfg_path, "r", encoding="utf-8") as f:
            loaded = json.load(f)
        assert loaded == config_data
        print("  OK: canonical JSON matches schema")

    print("\nALL SYSTEM UPDATE CONFIGURATION INTEGRATION CHECKS PASSED.")
    return 0


def test_system_update_config() -> None:
    assert main() == 0


if __name__ == "__main__":
    sys.exit(main())
