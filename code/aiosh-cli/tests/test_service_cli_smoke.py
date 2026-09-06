#!/usr/bin/env python3
"""CLI Smoke & Invariant Test for Init & Service Supervision (T-01323..T-01325)."""

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


def parse_json_output(res: subprocess.CompletedProcess) -> dict | list:
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)


def test_service_help():
    res = run_aiosh("service", "--help")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    assert "aiosh service" in res.stdout
    assert "validate" in res.stdout
    assert "list" in res.stdout
    assert "show" in res.stdout
    assert "action" in res.stdout
    assert "order" in res.stdout
    print("PASS: aiosh service --help")


def test_service_unknown_subcommand():
    res = run_aiosh("service", "unknown_cmd")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"
    print("PASS: aiosh service unknown_cmd returns 2")


def test_service_validate():
    # Valid name
    res_valid = run_aiosh("service", "validate", "--name", "aios-securityd.service")
    assert res_valid.returncode == 0, f"Unexpected returncode: {res_valid.stderr}"

    # Valid name json
    res_valid_json = run_aiosh("service", "validate", "--name", "aios-securityd.service", "--json")
    assert res_valid_json.returncode == 0
    data = parse_json_output(res_valid_json)
    assert data["code"] == 0
    assert data["data"]["valid"] is True

    # Invalid name
    res_invalid = run_aiosh("service", "validate", "--name", "bad/name.service")
    assert res_invalid.returncode == 2

    # Missing flags
    res_missing = run_aiosh("service", "validate")
    assert res_missing.returncode == 2

    print("PASS: aiosh service validate (name and json)")


def test_service_list():
    res = run_aiosh("service", "list")
    assert res.returncode == 0, f"Unexpected returncode: {res.stderr}"
    assert "AIOS Service Store" in res.stdout

    res_json = run_aiosh("service", "list", "--json")
    assert res_json.returncode == 0
    data = parse_json_output(res_json)
    assert isinstance(data, list)
    assert len(data) >= 5
    names = [s["name"] for s in data]
    assert "aios-securityd.service" in names
    assert "auditd.service" in names

    # Filter by state
    res_state = run_aiosh("service", "list", "--state", "active", "--json")
    assert res_state.returncode == 0

    # Bad state
    res_bad_state = run_aiosh("service", "list", "--state", "bogus_state")
    assert res_bad_state.returncode == 2

    print("PASS: aiosh service list (prose, json, filters)")


def test_service_show_and_status():
    res = run_aiosh("service", "show", "auditd.service")
    assert res.returncode == 0, f"Unexpected returncode: {res.stderr}"
    assert "auditd.service" in res.stdout

    # Show json
    res_json = run_aiosh("service", "show", "auditd.service", "--json")
    assert res_json.returncode == 0
    data = parse_json_output(res_json)
    assert data["code"] == 0
    assert data["data"]["service"]["name"] == "auditd.service"

    # Status alias
    res_status = run_aiosh("service", "status", "auditd.service", "--json")
    assert res_status.returncode == 0
    status_data = parse_json_output(res_status)
    assert status_data["code"] == 0
    assert status_data["data"]["service"]["name"] == "auditd.service"

    # Missing name
    res_missing = run_aiosh("service", "show")
    assert res_missing.returncode == 2

    # Not found
    res_not_found = run_aiosh("service", "show", "nonexistent.service")
    assert res_not_found.returncode == 1

    print("PASS: aiosh service show and status (prose, json, not found)")


def test_service_action_and_shortcuts():
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        temp_store = f.name

    try:
        # Seed initial store by writing default service store JSON
        res_list = run_aiosh("service", "list", "--json")
        services = parse_json_output(res_list)
        statuses = {}
        for s in services:
            statuses[s["name"]] = {
                "name": s["name"],
                "state": "active",
                "startup_mode": s["startup_mode"],
                "pid": 1234,
                "health": {"healthy": True, "restarts": 0},
                "started_at": "2026-09-06T00:00:00Z",
            }
        store_data = {
            "services": {s["name"]: s for s in services},
            "statuses": statuses,
        }
        with open(temp_store, "w", encoding="utf-8") as f:
            json.dump(store_data, f)

        # Now run action stop
        res_stop = run_aiosh(
            "service", "action", "auditd.service", "stop",
            "--store", temp_store, "--json"
        )
        assert res_stop.returncode == 0, f"Stop failed: {res_stop.stderr}"
        data_stop = parse_json_output(res_stop)
        assert data_stop["code"] == 0
        assert data_stop["data"]["new_state"] == "inactive"

        # Direct shortcut: start
        res_start = run_aiosh(
            "service", "start", "auditd.service",
            "--store", temp_store, "--json"
        )
        assert res_start.returncode == 0, f"Start shortcut failed: {res_start.stderr}"
        data_start = parse_json_output(res_start)
        assert data_start["code"] == 0
        assert data_start["data"]["new_state"] == "active"

        # Direct shortcut: restart
        res_restart = run_aiosh(
            "service", "restart", "auditd.service",
            "--store", temp_store, "--json"
        )
        assert res_restart.returncode == 0, f"Restart shortcut failed: {res_restart.stderr}"

        # Direct shortcut: reload
        res_reload = run_aiosh(
            "service", "reload", "auditd.service",
            "--store", temp_store, "--json"
        )
        assert res_reload.returncode == 0, f"Reload shortcut failed: {res_reload.stderr}"

        # Negative: bad action
        res_bad = run_aiosh("service", "action", "auditd.service", "bad_action")
        assert res_bad.returncode == 2

        # Negative: missing args
        res_missing_args = run_aiosh("service", "start")
        assert res_missing_args.returncode == 2

        print("PASS: aiosh service action and direct shortcuts (start/stop/restart/reload)")
    finally:
        if os.path.exists(temp_store):
            os.remove(temp_store)


def test_service_order():
    res = run_aiosh("service", "order", "aios-securityd.service", "--json")
    assert res.returncode == 0, f"Order failed: {res.stderr}"
    data = parse_json_output(res)
    assert data["code"] == 0
    order = data["data"]["order"]
    assert "aios-securityd.service" in order
    assert "auditd.service" in order
    assert "dbus.service" in order
    # auditd and dbus must come before aios-securityd
    idx_target = order.index("aios-securityd.service")
    idx_auditd = order.index("auditd.service")
    idx_dbus = order.index("dbus.service")
    assert idx_auditd < idx_target
    assert idx_dbus < idx_target

    # Negative: nonexistent service
    res_not_found = run_aiosh("service", "order", "nonexistent.service")
    assert res_not_found.returncode == 1

    # Negative: missing name
    res_missing = run_aiosh("service", "order")
    assert res_missing.returncode == 2

    print("PASS: aiosh service order (valid topological plan and negative tests)")


def main() -> int:
    test_service_help()
    test_service_unknown_subcommand()
    test_service_validate()
    test_service_list()
    test_service_show_and_status()
    test_service_action_and_shortcuts()
    test_service_order()
    print("\nALL SERVICE CLI SMOKE TESTS PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
