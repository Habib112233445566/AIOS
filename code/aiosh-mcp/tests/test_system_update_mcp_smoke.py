#!/usr/bin/env python3
"""Cross-Surface Integration Smoke Test for System Update MCP Surface (T-01936).

Proves that all 6 System Update MCP tools are exposed via JSON-RPC,
adhere to schema contracts, enforce bounds and security sanitization, and achieve
cross-surface state parity with the operator CLI.

Covers:
- Tool registration: All 6 `aios.update.*` tools advertised via tools/list.
- Status and Slots discovery: `aios.update.status` and `aios.update.slots`.
- Input validation: Refusal of malformed manifests, missing parameters, and paths with control chars.
- State transitions: `aios.update.check`, `aios.update.confirm`, and `aios.update.rollback`.
- Cross-surface parity: Comparing MCP tool responses with operator CLI output on shared state directory.

Run standalone:
    python code/aiosh-mcp/tests/test_system_update_mcp_smoke.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

UPDATE_TOOLS = (
    "aios.update.status",
    "aios.update.slots",
    "aios.update.check",
    "aios.update.apply",
    "aios.update.confirm",
    "aios.update.rollback",
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


def get_cli_binary() -> str:
    return _find_binary(["aiosh.exe", "aiosh"])


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
        data = res["structuredContent"]["result"]
    elif "content" in res and res["content"]:
        try:
            data = json.loads(res["content"][0]["text"])
        except Exception:
            data = {"ok": False, "text": res["content"][0]["text"]}
    else:
        data = res
    return data


def run_cli(args: list[str], timeout_s: int = 15) -> tuple[int, str, str]:
    cmd = [get_cli_binary()] + args
    p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout_s)
    return p.returncode, p.stdout, p.stderr


def main() -> int:
    print("=== System Update MCP Cross-Surface Smoke Test ===")

    # 1. Tool advertisement
    print("[1] Verifying tools/list exposes all 6 update tools...")
    list_payload = {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}
    list_resp = run_mcp(list_payload)
    tools = {t["name"] for t in list_resp.get("result", {}).get("tools", [])}
    for t in UPDATE_TOOLS:
        if t not in tools:
            print(f"FAIL: Tool {t} not found in tools/list response")
            return 1
        print(f"  OK: found {t}")

    with tempfile.TemporaryDirectory() as td:
        state_dir = os.path.join(td, "update_state")
        os.makedirs(state_dir, exist_ok=True)

        # 2. Input validation & path hygiene
        print("[2] Testing path hygiene & input bounds...")
        bad_path_res = call_mcp_tool("aios.update.status", {"state_dir": f"{state_dir}\x07bad"})
        assert bad_path_res.get("ok") is False, "Expected bad path with control char to fail"
        print("  OK: control characters in state_dir rejected")

        empty_check = call_mcp_tool("aios.update.check", {"state_dir": state_dir})
        assert empty_check.get("ok") is False, "Expected check without manifest to fail"
        print("  OK: missing manifest in check rejected")

        long_ver = "v" * 65
        long_ver_res = call_mcp_tool("aios.update.confirm", {"state_dir": state_dir, "version": long_ver})
        assert long_ver_res.get("ok") is False, "Expected long version to fail"
        print("  OK: version > 64 chars rejected")

        # 3. Initial Status & Slots queries
        print("[3] Testing initial status and slots queries...")
        status_res = call_mcp_tool("aios.update.status", {"state_dir": state_dir})
        assert status_res.get("ok") is True, f"Expected ok: true, got {status_res}"
        status_data = status_res.get("data", {})
        assert status_data.get("state") == "idle", f"Expected state: idle, got {status_data}"
        assert status_data.get("active_slot") == "slot_a", f"Expected active_slot: slot_a, got {status_data}"
        print(f"  OK: status={status_data.get('state')}, active_slot={status_data.get('active_slot')}")

        slots_res = call_mcp_tool("aios.update.slots", {"state_dir": state_dir})
        assert slots_res.get("ok") is True, f"Expected ok: true, got {slots_res}"
        slots_data = slots_res.get("data", {})
        assert slots_data.get("current_slot") == "slot_a", f"Expected slot_a, got {slots_data}"
        assert slots_data.get("target_slot") == "slot_b", f"Expected slot_b, got {slots_data}"
        print(f"  OK: current={slots_data.get('current_slot')}, target={slots_data.get('target_slot')}")

        # 4. State transitions: check
        print("[4] Testing aios.update.check with inline manifest...")
        valid_sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        manifest_data = {
            "update_id": "upd-2026-09-20-01",
            "version": "2.0.0",
            "channel": "stable",
            "artifacts": [
                {
                    "target": "rootfs",
                    "file_name": "rootfs.raw",
                    "sha256": valid_sha,
                    "size_bytes": 1024 * 1024,
                }
            ],
            "release_notes": "Security and stability release",
            "published_at": "2026-09-20T00:00:00Z",
        }
        check_res = call_mcp_tool(
            "aios.update.check",
            {"state_dir": state_dir, "manifest": manifest_data},
        )
        assert check_res.get("ok") is True, f"Expected ok: true, got {check_res}"
        check_data = check_res.get("data", {})
        assert check_data.get("target_version") == "2.0.0", f"Expected 2.0.0, got {check_data}"
        print(f"  OK: check successful, target_version={check_data.get('target_version')}")

        # Confirm status updated to downloading
        status_res2 = call_mcp_tool("aios.update.status", {"state_dir": state_dir})
        assert status_res2.get("ok") is True
        status_data2 = status_res2.get("data", {})
        assert status_data2.get("state") == "downloading"
        assert status_data2.get("target_version") == "2.0.0"
        print("  OK: status transitioned to downloading")

        # 5. Confirm in wrong state fails
        print("[5] Testing confirm in non-ReadyToReboot state...")
        confirm_fail = call_mcp_tool("aios.update.confirm", {"state_dir": state_dir})
        assert confirm_fail.get("ok") is False
        print("  OK: confirm rejected in downloading state")

        # 6. Manually set state to ReadyToReboot to test confirm & rollback
        print("[6] Testing confirm and rollback in ReadyToReboot state...")
        update_status_file = os.path.join(state_dir, "update_status.json")
        with open(update_status_file, "r") as f:
            st = json.load(f)
        st["state"] = "ready_to_reboot"
        with open(update_status_file, "w") as f:
            json.dump(st, f)

        confirm_res = call_mcp_tool("aios.update.confirm", {"state_dir": state_dir, "version": "2.0.0"})
        assert confirm_res.get("ok") is True, f"Expected confirm to succeed, got {confirm_res}"
        confirm_data = confirm_res.get("data", {})
        assert confirm_data.get("confirmed_version") == "2.0.0"
        print(f"  OK: confirm succeeded, confirmed_version={confirm_data.get('confirmed_version')}")

        # Reset to ReadyToReboot for rollback test
        with open(update_status_file, "r") as f:
            st = json.load(f)
        st["state"] = "ready_to_reboot"
        with open(update_status_file, "w") as f:
            json.dump(st, f)

        rollback_res = call_mcp_tool("aios.update.rollback", {"state_dir": state_dir})
        assert rollback_res.get("ok") is True, f"Expected rollback to succeed, got {rollback_res}"
        rollback_data = rollback_res.get("data", {})
        assert rollback_data.get("restored_slot") == "slot_a"
        print(f"  OK: rollback succeeded, restored_slot={rollback_data.get('restored_slot')}")

        # 7. Cross-Surface Parity with CLI
        print("[7] Testing cross-surface parity between CLI and MCP...")
        rc, out, err = run_cli(["update", "status", "--state-dir", state_dir, "--json"])
        assert rc == 0, f"CLI update status failed: {err}"
        cli_env = json.loads(out.strip())
        cli_data = cli_env.get("data", {})

        mcp_json = call_mcp_tool("aios.update.status", {"state_dir": state_dir})
        mcp_status_data = mcp_json.get("data", {})
        assert cli_data.get("active_slot") == mcp_status_data.get("active_slot"), (
            f"Active slot mismatch: CLI {cli_data.get('active_slot')} vs MCP {mcp_status_data.get('active_slot')}"
        )
        assert cli_data.get("state") == mcp_status_data.get("state"), (
            f"State mismatch: CLI {cli_data.get('state')} vs MCP {mcp_status_data.get('state')}"
        )
        print("  OK: CLI and MCP agree on state and active_slot")

    print("\nALL 7 SYSTEM UPDATE MCP SMOKE CHECKS PASSED.")
    return 0


def test_system_update_mcp() -> None:
    assert main() == 0


if __name__ == "__main__":
    sys.exit(main())
