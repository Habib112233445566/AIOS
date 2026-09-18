#!/usr/bin/env python3
"""CLI Smoke & Invariant Test for User Session Bootstrap (T-01406)."""

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
    bin_path = get_binary_path()
    cmd = [bin_path, *args]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    return res


def parse_json_output(res: subprocess.CompletedProcess) -> dict:
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)


def test_session_help():
    res = run_aiosh("session", "--help")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    assert "aiosh session" in res.stdout
    assert "validate" in res.stdout
    print("PASS: aiosh session --help")


def test_session_unknown_subcommand():
    res = run_aiosh("session", "unknown_cmd")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"
    print("PASS: aiosh session unknown_cmd returns 2")


def test_session_validate():
    # 1. Valid session ID
    res = run_aiosh("session", "validate", "--id", "sess-01")
    assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
    assert "VALID" in res.stdout
    assert "sess-01" in res.stdout

    # 2. Valid session ID with JSON
    res_json = run_aiosh("session", "validate", "--id", "sess-01", "--json")
    assert res_json.returncode == 0
    data = parse_json_output(res_json)
    assert data["code"] == 0
    assert data["data"]["valid"] is True
    assert data["data"]["session_id"] == "sess-01"

    # 3. Invalid session ID
    res_bad = run_aiosh("session", "validate", "--id", "../evil", "--json")
    assert res_bad.returncode != 0
    data_bad = parse_json_output(res_bad)
    assert data_bad["code"] == 2
    assert data_bad["data"]["valid"] is False
    assert data_bad["error"] is not None

    # 4. Valid username
    res_u = run_aiosh("session", "validate", "--user", "kali")
    assert res_u.returncode == 0
    assert "VALID" in res_u.stdout

    # 5. Invalid username (uppercase rejected)
    res_u_bad = run_aiosh("session", "validate", "--user", "Kali", "--json")
    assert res_u_bad.returncode != 0
    data_u_bad = parse_json_output(res_u_bad)
    assert data_u_bad["code"] == 2
    assert data_u_bad["data"]["valid"] is False

    # 6. Valid Spec via JSON inline
    valid_spec = {
        "session_id": "sess-01",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "x11",
        "session_class": "user",
        "seat": "seat0",
        "vtnr": 7,
        "display": ":0",
        "remote_host": None,
        "environment": {}
    }
    res_spec = run_aiosh("session", "validate", "--spec", json.dumps(valid_spec), "--json")
    assert res_spec.returncode == 0
    data_spec = parse_json_output(res_spec)
    assert data_spec["code"] == 0
    assert data_spec["data"]["valid"] is True

    # 7. Invalid Spec (missing display for X11)
    bad_spec = dict(valid_spec)
    bad_spec["display"] = None
    res_bad_spec = run_aiosh("session", "validate", "--spec", json.dumps(bad_spec), "--json")
    assert res_bad_spec.returncode != 0
    data_bad_spec = parse_json_output(res_bad_spec)
    assert data_bad_spec["code"] == 2
    assert data_bad_spec["data"]["valid"] is False

    print("PASS: aiosh session validate (id, user, spec, and json)")


def test_session_hardening():
    # 1. Reject oversized payload (>1 MiB) via file
    with tempfile.NamedTemporaryFile("w", delete=False) as f:
        f.write("x" * (1024 * 1024 + 50))
        tmp_path = f.name
    try:
        res_huge = run_aiosh("session", "validate", "--spec", tmp_path, "--json")
        assert res_huge.returncode != 0
        data_huge = parse_json_output(res_huge)
        assert data_huge["code"] == 2
        assert data_huge["error"]["code"] == "PAYLOAD_TOO_LARGE"
    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)

    # 2. Reject malformed JSON
    res_malformed = run_aiosh("session", "validate", "--spec", "{not:valid:json}", "--json")
    assert res_malformed.returncode != 0
    data_malformed = parse_json_output(res_malformed)
    assert data_malformed["code"] == 2
    assert data_malformed["error"]["code"] == "JSON_PARSE_ERROR"

    # 3. Missing argument returns MISSING_ARGUMENTS
    res_missing = run_aiosh("session", "validate", "--json")
    assert res_missing.returncode != 0
    data_missing = parse_json_output(res_missing)
    assert data_missing["code"] == 2
    assert data_missing["error"]["code"] == "MISSING_ARGUMENTS"

    # 4. Missing arguments for session show
    res_show_missing = run_aiosh("session", "show", "--json")
    assert res_show_missing.returncode == 2
    data_show_missing = parse_json_output(res_show_missing)
    assert data_show_missing["code"] == 2
    assert data_show_missing["error"]["code"] == "MISSING_ARGUMENTS"

    # 5. Missing arguments for session action
    res_act_missing = run_aiosh("session", "action", "--json")
    assert res_act_missing.returncode == 2
    data_act_missing = parse_json_output(res_act_missing)
    assert data_act_missing["code"] == 2
    assert data_act_missing["error"]["code"] == "MISSING_ARGUMENTS"

    # 6. Invalid action name for session action
    res_bad_act = run_aiosh("session", "action", "greeter-seat0", "invalid_action_xyz", "--json")
    assert res_bad_act.returncode == 2
    data_bad_act = parse_json_output(res_bad_act)
    assert data_bad_act["code"] == 2
    assert data_bad_act["error"]["code"] == "INVALID_ACTION"

    # 7. Failed action (unlock on active session) returns explicit ACTION_FAILED
    res_act_fail = run_aiosh("session", "action", "greeter-seat0", "unlock", "--json")
    assert res_act_fail.returncode == 1
    data_act_fail = parse_json_output(res_act_fail)
    assert data_act_fail["code"] == 1
    assert data_act_fail["error"]["code"] == "ACTION_FAILED"

    # 8. Invalid store path with control characters and excessive length
    res_bad_ctrl = run_aiosh("session", "list", "--store", "bad\x01store", "--json")
    assert res_bad_ctrl.returncode == 2
    data_bad_ctrl = parse_json_output(res_bad_ctrl)
    assert data_bad_ctrl["code"] == 2
    assert data_bad_ctrl["error"]["code"] == "INVALID_ARGUMENT"

    res_huge_store = run_aiosh("session", "list", "--store", "s" * 1025, "--json")
    assert res_huge_store.returncode == 2
    data_huge_store = parse_json_output(res_huge_store)
    assert data_huge_store["code"] == 2
    assert data_huge_store["error"]["code"] == "INVALID_ARGUMENT"

    # 9. Invalid limit parameter
    res_bad_limit = run_aiosh("session", "list", "--limit", "0", "--json")
    assert res_bad_limit.returncode == 2
    data_bad_limit = parse_json_output(res_bad_limit)
    assert data_bad_limit["code"] == 2
    assert data_bad_limit["error"]["code"] == "INVALID_ARGUMENT"

    print("PASS: aiosh session hardening (payload limits, json parse, missing args, action envelopes, store & limit checks)")


def test_session_list_show_action():
    # 1. Test list default canonical greeter
    res_list = run_aiosh("session", "list", "--json")
    assert res_list.returncode == 0, f"List failed: {res_list.stderr}"
    data_list = parse_json_output(res_list)
    assert data_list["code"] == 0
    assert data_list["data"]["count"] >= 1
    sessions = data_list["data"]["sessions"]
    assert any(s["session_id"] == "greeter-seat0" for s in sessions)

    # 2. Test show greeter-seat0
    res_show = run_aiosh("session", "show", "greeter-seat0", "--json")
    assert res_show.returncode == 0, f"Show failed: {res_show.stderr}"
    data_show = parse_json_output(res_show)
    assert data_show["code"] == 0
    assert data_show["data"]["status"]["username"] == "lightdm"
    assert data_show["data"]["status"]["state"] == "active"

    # 3. Test action: Lock greeter session
    res_lock = run_aiosh("session", "action", "greeter-seat0", "lock", "--json")
    assert res_lock.returncode == 0, f"Action lock failed: {res_lock.stderr}"
    data_lock = parse_json_output(res_lock)
    assert data_lock["code"] == 0
    assert data_lock["data"]["new_state"] == "locked"

    # 4. Test show non-existent session -> code 1
    res_missing = run_aiosh("session", "show", "non-existent-session", "--json")
    assert res_missing.returncode != 0

    print("PASS: aiosh session list, show, action")


def test_session_scaffolded_commands():
    # 1. Status alias for show
    res_status = run_aiosh("session", "status", "greeter-seat0", "--json")
    assert res_status.returncode == 0
    data_status = parse_json_output(res_status)
    assert data_status["code"] == 0
    assert data_status["data"]["status"]["session_id"] == "greeter-seat0"

    # 2. Lock shortcut alias (in-memory)
    res_lock = run_aiosh("session", "lock", "greeter-seat0", "--json")
    assert res_lock.returncode == 0
    data_lock = parse_json_output(res_lock)
    assert data_lock["code"] == 0
    assert data_lock["data"]["new_state"] == "locked"

    # 3. Persistent Lock -> Unlock sequence with --store
    with tempfile.NamedTemporaryFile("w", delete=False, suffix=".json") as f:
        tmp_store = f.name
    try:
        res_l = run_aiosh("session", "lock", "greeter-seat0", "--store", tmp_store, "--json")
        assert res_l.returncode == 0
        res_unlock = run_aiosh("session", "unlock", "greeter-seat0", "--store", tmp_store, "--json")
        assert res_unlock.returncode == 0
        data_unlock = parse_json_output(res_unlock)
        assert data_unlock["code"] == 0
        assert data_unlock["data"]["new_state"] == "active"
    finally:
        if os.path.exists(tmp_store):
            os.remove(tmp_store)

    # 4. Activate shortcut alias
    res_act = run_aiosh("session", "activate", "greeter-seat0", "--json")
    assert res_act.returncode == 0
    data_act = parse_json_output(res_act)
    assert data_act["code"] == 0

    # 5. Create missing argument returns 2
    res_create_missing = run_aiosh("session", "create", "--json")
    assert res_create_missing.returncode == 2
    data_create_missing = parse_json_output(res_create_missing)
    assert data_create_missing["code"] == 2
    assert data_create_missing["error"]["code"] == "MISSING_ARGUMENTS"

    # 6. Create valid session from inline spec JSON
    valid_new_spec = {
        "session_id": "sess-created-01",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "x11",
        "session_class": "user",
        "seat": "seat0",
        "vtnr": 7,
        "display": ":0",
        "remote_host": None,
        "environment": {}
    }
    res_create = run_aiosh("session", "create", json.dumps(valid_new_spec), "--json")
    assert res_create.returncode == 0
    data_create = parse_json_output(res_create)
    assert data_create["code"] == 0
    assert data_create["data"]["session_id"] == "sess-created-01"
    assert data_create["data"]["new_state"] == "initializing"

    # 7. Create invalid spec returns 2
    bad_spec = dict(valid_new_spec)
    bad_spec["session_id"] = "invalid/id"
    res_bad_create = run_aiosh("session", "create", json.dumps(bad_spec), "--json")
    assert res_bad_create.returncode == 2
    data_bad_create = parse_json_output(res_bad_create)
    assert data_bad_create["code"] == 2
    assert data_bad_create["error"]["code"] == "VALIDATION_FAILED"

    print("PASS: aiosh session commands (status, shortcuts, create)")


def main() -> int:
    print("=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===")
    test_session_help()
    test_session_unknown_subcommand()
    test_session_validate()
    test_session_list_show_action()
    test_session_scaffolded_commands()
    test_session_hardening()
    print("\nALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
