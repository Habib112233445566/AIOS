#!/usr/bin/env python3
"""Standalone test runner for Phase 1 Init & Service Supervision subsystem.

Criteria:
  SS1: service data model integrity & invariants (SS1..SS5)
  SS2: service CLI surface commands & options (validate, list, show/status, action, order)
  SS3: service MCP tool surface (validate, list, get, action, order, config)
  SS4: service core service lifecycle, FSM & dependency ordering (CS1..CS5)
  SS5: service configuration subsystem invariants, precedence & sizing (SC1..SC7)
  SS6: service automated integration tests (ST1..ST5)
  SS7: service security policy invariants & evaluation (SP1..SP6)
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def _run_cargo_test(extra_args: list[str], criterion: str, description: str) -> bool:
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"),
        *extra_args,
    ]
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    except subprocess.TimeoutExpired:
        print(f"[-] {criterion} timed out after 120s", file=sys.stderr)
        return False
    except Exception as e:
        print(f"[-] {criterion} execution error: {e}", file=sys.stderr)
        return False

    if res.returncode != 0:
        print(f"[-] {criterion} cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print(f"[+] {criterion} {description}")
    return True


def test_ss1_data_model_integrity():
    return _run_cargo_test(
        ["--test", "test_service_data_model"],
        "SS1",
        "service data model integrity & invariants (SS1..SS5)",
    )


def test_ss2_cli_surface_commands():
    return _run_cargo_test(
        ["--bin", "aiosh", "test_cmd_service_flow"],
        "SS2",
        "service CLI surface commands & options (validate, list, show/status, action, order)",
    )


def test_ss3_mcp_surface_tools():
    return _run_cargo_test(
        ["--bin", "aiosh-mcp", "test_mcp_service_tools"],
        "SS3",
        "service MCP tool surface (validate, list, get, action, order)",
    )


def test_ss4_core_service_lifecycle():
    return _run_cargo_test(
        ["--test", "test_service_service"],
        "SS4",
        "service core service lifecycle, FSM & dependency ordering (CS1..CS5)",
    )


def test_ss5_configuration_subsystem():
    return _run_cargo_test(
        ["--test", "test_service_config"],
        "SS5",
        "service configuration subsystem invariants, precedence & sizing (SC1..SC7)",
    )


def test_ss6_automated_integration():
    return _run_cargo_test(
        ["--test", "test_service_automated"],
        "SS6",
        "service automated integration tests (ST1..ST5)",
    )


def test_ss7_security_policy():
    return _run_cargo_test(
        ["--test", "test_service_policy"],
        "SS7",
        "service security policy invariants & evaluation (SP1..SP6)",
    )


def main():
    checks = [
        test_ss1_data_model_integrity,
        test_ss2_cli_surface_commands,
        test_ss3_mcp_surface_tools,
        test_ss4_core_service_lifecycle,
        test_ss5_configuration_subsystem,
        test_ss6_automated_integration,
        test_ss7_security_policy,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: service_suites criteria (SS1..SS7)")
        return 0
    else:
        print("\nFAIL: service_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
