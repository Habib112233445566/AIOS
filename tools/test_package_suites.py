#!/usr/bin/env python3
"""Standalone test runner for Phase 1 Package Management subsystem.

Criteria:
  PM1: package data model integrity & invariants (PM1..PM5)
  PM2: package core service integrity & invariants (CS1..CS5)
  PM3: package CLI surface commands & options (validate/list/show/search/plan/apply)
  PM4: package MCP tool surface (validate/list/get/plan/search/apply)
  PM5: package configuration resolution & invariants (PC1..PC6)
  PM6: package automated integration test matrix (PT1..PT6)
  PM7: package security policy evaluation & invariants (PP1..PP6)
  PM8: package observability telemetry report & invariants (PO1..PO6)
  PM9: package documentation guide & invariants (D1..D5)
  PM10: package recovery & validation integrity (RV1..RV4)
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


def test_pm1_data_model_integrity():
    return _run_cargo_test(
        ["--test", "test_package_data_model"],
        "PM1",
        "package data model integrity & invariants (PM1..PM5)",
    )


def test_pm2_core_service_integrity():
    return _run_cargo_test(
        ["--test", "test_package_service"],
        "PM2",
        "package core service integrity & invariants (CS1..CS5)",
    )


def test_pm3_cli_surface_commands():
    return _run_cargo_test(
        ["--bin", "aiosh", "test_cmd_package_flow"],
        "PM3",
        "package CLI surface commands & options (validate/list/show/search/plan/apply)",
    )


def test_pm4_mcp_surface_tools():
    return _run_cargo_test(
        ["--bin", "aiosh-mcp", "test_mcp_package_tools"],
        "PM4",
        "package MCP tool surface (validate/list/get/plan/search/apply)",
    )


def test_pm5_configuration_resolution():
    return _run_cargo_test(
        ["--test", "test_package_config"],
        "PM5",
        "package configuration resolution & invariants (PC1..PC6)",
    )


def test_pm6_automated_integration():
    return _run_cargo_test(
        ["--test", "test_package_automated"],
        "PM6",
        "package automated integration test matrix (PT1..PT6)",
    )


def test_pm7_security_policy():
    return _run_cargo_test(
        ["--test", "test_package_policy"],
        "PM7",
        "package security policy evaluation & invariants (PP1..PP6)",
    )


def test_pm8_observability():
    return _run_cargo_test(
        ["--test", "test_package_observability"],
        "PM8",
        "package observability telemetry report & invariants (PO1..PO6)",
    )


def test_pm9_documentation():
    cmd = [sys.executable, str(ROOT / "tools" / "test_package_doc.py")]
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=30)
    except subprocess.TimeoutExpired:
        print("[-] PM9 timed out after 30s", file=sys.stderr)
        return False
    except Exception as e:
        print(f"[-] PM9 execution error: {e}", file=sys.stderr)
        return False

    if res.returncode != 0:
        print(f"[-] PM9 documentation test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] PM9 package documentation guide & invariants (D1..D5)")
    return True


def test_pm10_recovery_validation():
    return _run_cargo_test(
        ["--test", "test_package_recovery"],
        "PM10",
        "package recovery & validation integrity (RV1..RV4)",
    )


def main():
    checks = [
        test_pm1_data_model_integrity,
        test_pm2_core_service_integrity,
        test_pm3_cli_surface_commands,
        test_pm4_mcp_surface_tools,
        test_pm5_configuration_resolution,
        test_pm6_automated_integration,
        test_pm7_security_policy,
        test_pm8_observability,
        test_pm9_documentation,
        test_pm10_recovery_validation,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: package_suites criteria (PM1..PM10)")
        return 0
    else:
        print("\nFAIL: package_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
