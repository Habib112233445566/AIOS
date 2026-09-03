#!/usr/bin/env python3
"""Standalone test runner for Phase 1 Distro Selection & Justification subsystem.

Criteria:
  D1: distro data model integrity & evaluation scoring
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


def test_d1_data_model_integrity():
    return _run_cargo_test(
        ["--lib", "distro::tests::test_distro_profile_validation_and_defaults"],
        "D1",
        "distro data model integrity & validation invariants",
    )


def test_d2_core_service_suite():
    return _run_cargo_test(
        ["--lib", "distro_service::tests::test_distro_store_lifecycle_and_evaluations"],
        "D2",
        "distro store lifecycle, registry querying & persistence",
    )


def test_d3_cli_surface():
    return _run_cargo_test(
        ["--bin", "aiosh", "test_cmd_distro_flow"],
        "D3",
        "distro CLI surface commands & options (list/show/evaluate/recommend)",
    )


def test_d4_mcp_surface():
    return _run_cargo_test(
        ["--bin", "aiosh-mcp", "test_mcp_distro_tools"],
        "D4",
        "distro MCP tools dispatch & execution (list/show/evaluate/recommend)",
    )


def test_d5_configuration_subsystem():
    return _run_cargo_test(
        ["--lib", "distro_config::tests"],
        "D5",
        "distro configuration resolution & hardening invariants",
    )


def main():
    checks = [
        test_d1_data_model_integrity,
        test_d2_core_service_suite,
        test_d3_cli_surface,
        test_d4_mcp_surface,
        test_d5_configuration_subsystem,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: distro_suites criteria (D1..D5)")
        return 0
    else:
        print("\nFAIL: distro_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
