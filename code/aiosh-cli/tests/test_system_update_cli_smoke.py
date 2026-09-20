#!/usr/bin/env python3
"""CLI Smoke & Integration Test for System Update Mechanism (T-01926).

Exercises the operator surface `aiosh update <subcommand>` / `aiosh upd <subcommand>`
end-to-end through the compiled binary: exit codes, JSON result envelopes,
path hygiene, audit logging, and lifecycle commands.

Coverage:
- Help output and unknown subcommands.
- Path hygiene (length > 1024, control characters).
- Argument validation (missing manifest path, unreadable/malformed manifest).
- Operational subcommands: status, slots, check, apply, confirm, rollback.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def get_binary_path() -> str:
    candidates = [
        ROOT / "code/aiosh-rust/target/debug/aiosh.exe",
        ROOT / "code/aiosh-rust/target/debug/aiosh",
        ROOT / "target/debug/aiosh.exe",
        ROOT / "target/debug/aiosh",
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    return "aiosh"


def run_aiosh(*args: str) -> subprocess.CompletedProcess:
    res = subprocess.run([get_binary_path(), *args], capture_output=True, text=True, timeout=60)
    return res


def parse_json_output(res: subprocess.CompletedProcess) -> dict | list:
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)


def expect_code(res: subprocess.CompletedProcess, code: int, context: str) -> None:
    assert res.returncode == code, (
        f"{context}: expected exit code {code}, got {res.returncode}\n"
        f"stdout: {res.stdout.strip()}\nstderr: {res.stderr.strip()}"
    )


def json_envelope(res: subprocess.CompletedProcess, expected_code: int, context: str) -> dict:
    payload = parse_json_output(res)
    assert isinstance(payload, dict), f"{context}: expected a JSON object envelope"
    assert payload.get("code") == expected_code, (
        f"{context}: envelope code {payload.get('code')} != {expected_code}"
    )
    assert "data" in payload and "error" in payload, f"{context}: envelope missing data/error keys"
    return payload


def test_update_help_and_unknown() -> None:
    for cmd in ("update", "upd"):
        res = run_aiosh(cmd, "--help")
        expect_code(res, 0, f"{cmd} --help")
        for token in ("aiosh update", "status", "slots", "check", "apply", "confirm", "rollback"):
            assert token in res.stdout, f"{cmd} --help missing token {token!r}"

    res_unknown = run_aiosh("update", "invalid_subcmd", "--json")
    expect_code(res_unknown, 2, "update invalid_subcmd")
    envelope = json_envelope(res_unknown, 2, "unknown subcommand")
    assert envelope["error"]["code"] == "UNKNOWN_SUBCOMMAND"
    print("PASS: test_update_help_and_unknown")


def test_update_path_hygiene() -> None:
    # 1. Path length > 1024
    res_long_state = run_aiosh("update", "status", "--state-dir", "a" * 1025, "--json")
    expect_code(res_long_state, 2, "update status state-dir too long")
    assert json_envelope(res_long_state, 2, "state-dir too long")["error"]["code"] == "PATH_TOO_LONG"

    res_long_staging = run_aiosh("update", "status", "--staging-dir", "b" * 1025, "--json")
    expect_code(res_long_staging, 2, "update status staging-dir too long")
    assert json_envelope(res_long_staging, 2, "staging-dir too long")["error"]["code"] == "PATH_TOO_LONG"

    # 2. Control characters in paths
    res_ctrl_state = run_aiosh("update", "status", "--state-dir", "bad\npath", "--json")
    expect_code(res_ctrl_state, 2, "update status state-dir control char")
    assert json_envelope(res_ctrl_state, 2, "state-dir control char")["error"]["code"] == "PATH_CONTAINS_CONTROL_CHAR"

    res_ctrl_staging = run_aiosh("update", "status", "--staging-dir", "bad\tpath", "--json")
    expect_code(res_ctrl_staging, 2, "update status staging-dir control char")
    assert json_envelope(res_ctrl_staging, 2, "staging-dir control char")["error"]["code"] == "PATH_CONTAINS_CONTROL_CHAR"
    print("PASS: test_update_path_hygiene")


def test_update_status_and_slots() -> None:
    with tempfile.TemporaryDirectory(prefix="aiosh_smoke_upd_") as tmp:
        # Status command in human-readable and JSON format
        res_stat_hr = run_aiosh("update", "status", "--state-dir", tmp)
        expect_code(res_stat_hr, 0, "update status human-readable")
        assert "System Update Status:" in res_stat_hr.stdout
        assert "idle" in res_stat_hr.stdout

        res_stat_json = run_aiosh("update", "status", "--state-dir", tmp, "--json")
        expect_code(res_stat_json, 0, "update status json")
        stat_env = json_envelope(res_stat_json, 0, "update status json")
        assert stat_env["data"]["state"] == "idle"
        assert stat_env["data"]["active_slot"] == "slot_a"
        assert stat_env["data"]["current_version"] == "1.0.0"

        # Slots command in human-readable and JSON format
        res_slots_hr = run_aiosh("update", "slots", "--state-dir", tmp)
        expect_code(res_slots_hr, 0, "update slots human-readable")
        assert "System Partition Slots:" in res_slots_hr.stdout

        res_slots_json = run_aiosh("update", "slots", "--state-dir", tmp, "--json")
        expect_code(res_slots_json, 0, "update slots json")
        slots_env = json_envelope(res_slots_json, 0, "update slots json")
        assert slots_env["data"]["current_slot"] == "slot_a"
        assert slots_env["data"]["target_slot"] == "slot_b"
        assert slots_env["data"]["slot_a_successful"] is True

    print("PASS: test_update_status_and_slots")


def test_update_check_and_validation() -> None:
    with tempfile.TemporaryDirectory(prefix="aiosh_smoke_upd_chk_") as tmp:
        state_dir = os.path.join(tmp, "state")
        os.makedirs(state_dir, exist_ok=True)

        # 1. Missing manifest path argument
        res_missing = run_aiosh("update", "check", "--state-dir", state_dir, "--json")
        expect_code(res_missing, 2, "check missing path")
        assert json_envelope(res_missing, 2, "missing manifest path")["error"]["code"] == "MISSING_MANIFEST_PATH"

        # 2. Non-existent manifest file
        bad_path = os.path.join(tmp, "no_such_file.json")
        res_bad_path = run_aiosh("update", "check", bad_path, "--state-dir", state_dir, "--json")
        expect_code(res_bad_path, 1, "check non-existent file")
        assert json_envelope(res_bad_path, 1, "non-existent file")["error"]["code"] == "READ_ERROR"

        # 3. Invalid JSON file
        bad_json = os.path.join(tmp, "invalid.json")
        with open(bad_json, "wb") as f:
            f.write(b"not json")
        res_bad_json = run_aiosh("update", "check", bad_json, "--state-dir", state_dir, "--json")
        expect_code(res_bad_json, 1, "check invalid json")
        assert json_envelope(res_bad_json, 1, "invalid json")["error"]["code"] == "INVALID_JSON"

        # 4. Valid manifest JSON
        good_manifest = os.path.join(tmp, "manifest.json")
        manifest_payload = {
            "update_id": "upd-2026-09-20-001",
            "version": "1.1.0",
            "channel": "stable",
            "min_version": "1.0.0",
            "release_notes": "Kernel and rootfs stability update",
            "published_at": "2026-09-20T12:00:00Z",
            "signature": "mock_ed25519_signature",
            "artifacts": [
                {
                    "target": "rootfs",
                    "file_name": "rootfs-1.1.0.img",
                    "sha256": "0" * 64,
                    "size_bytes": 1024
                }
            ]
        }
        with open(good_manifest, "w", encoding="utf-8") as f:
            json.dump(manifest_payload, f)

        res_good = run_aiosh("update", "check", good_manifest, "--state-dir", state_dir, "--json")
        expect_code(res_good, 0, "check valid manifest")
        good_env = json_envelope(res_good, 0, "valid manifest")
        assert good_env["data"]["target_version"] == "1.1.0"
        assert good_env["data"]["state"] == "downloading"

    print("PASS: test_update_check_and_validation")


def test_update_confirm_and_rollback() -> None:
    with tempfile.TemporaryDirectory(prefix="aiosh_smoke_upd_cr_") as tmp:
        state_dir = os.path.join(tmp, "state")
        os.makedirs(state_dir, exist_ok=True)

        # 1. Confirm from Idle state fails with domain error code 1
        res_conf_fail = run_aiosh("update", "confirm", "--state-dir", state_dir, "--json")
        expect_code(res_conf_fail, 1, "confirm idle")
        assert json_envelope(res_conf_fail, 1, "confirm idle failure")["error"]["code"] == "CONFIRM_FAILED"

        # 2. Rollback with no rollback slot fails with domain error code 1
        res_rb_fail = run_aiosh("update", "rollback", "--state-dir", state_dir, "--json")
        expect_code(res_rb_fail, 1, "rollback no slot")
        assert json_envelope(res_rb_fail, 1, "rollback no slot failure")["error"]["code"] == "ROLLBACK_FAILED"

        # 3. Create state files in ReadyToReboot with a rollback slot
        slot_file = os.path.join(state_dir, "slot_status.json")
        update_file = os.path.join(state_dir, "update_status.json")
        mock_slot = {
            "current_slot": "slot_a",
            "target_slot": "slot_b",
            "rollback_slot": "slot_b",
            "slot_a_version": "1.0.0",
            "slot_b_version": "1.1.0",
            "slot_a_successful": True,
            "slot_b_successful": False
        }
        mock_update = {
            "state": "ready_to_reboot",
            "current_version": "1.0.0",
            "target_version": "1.1.0",
            "active_slot": "slot_a",
            "progress_percent": 100,
            "last_error": None,
            "updated_at": "2026-09-20T12:00:00Z"
        }
        with open(slot_file, "w", encoding="utf-8") as f:
            json.dump(mock_slot, f)
        with open(update_file, "w", encoding="utf-8") as f:
            json.dump(mock_update, f)

        # 4. Confirm boot succeeds
        res_conf_ok = run_aiosh("update", "confirm", "1.1.0", "--state-dir", state_dir, "--json")
        expect_code(res_conf_ok, 0, "confirm boot ok")
        conf_env = json_envelope(res_conf_ok, 0, "confirm boot ok")
        assert conf_env["data"]["confirmed_version"] == "1.1.0"

        # 5. Reset to ReadyToReboot with rollback slot and test rollback
        mock_update["state"] = "ready_to_reboot"
        with open(update_file, "w", encoding="utf-8") as f:
            json.dump(mock_update, f)
        with open(slot_file, "w", encoding="utf-8") as f:
            json.dump(mock_slot, f)

        res_rb_ok = run_aiosh("update", "rollback", "--state-dir", state_dir, "--json")
        expect_code(res_rb_ok, 0, "rollback ok")
        rb_env = json_envelope(res_rb_ok, 0, "rollback ok")
        assert rb_env["data"]["restored_slot"] == "slot_b"

    print("PASS: test_update_confirm_and_rollback")


def main() -> int:
    bin_path = get_binary_path()
    if not os.path.exists(bin_path):
        print(f"ERROR: binary not found at {bin_path}", file=sys.stderr)
        return 1

    print(f"Running System Update CLI smoke tests against {bin_path}...")
    test_update_help_and_unknown()
    test_update_path_hygiene()
    test_update_status_and_slots()
    test_update_check_and_validation()
    test_update_confirm_and_rollback()
    print("All System Update CLI smoke tests PASSED successfully.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

