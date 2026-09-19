#!/usr/bin/env python3
"""Standalone test runner for the Phase 1 Filesystem Layout subsystem.

Criteria:
  FL1: filesystem layout data model & invariants (FL1..FL5)
  FL2: filesystem layout core service (store, probe, diff, fstab, persistence)
  FL3: filesystem layout CLI surface smoke & boundaries (cmd_fs_layout)
  FL4: filesystem layout CLI audit emission & escape-injection security proof (ADR-0035 / CWE-150)
  FL5: in-tree Rust unit tests for the CLI and MCP surfaces
  FL6: filesystem layout cross-surface CLI <-> MCP integration parity
  FL7: filesystem layout CLI hardening proof (non-regular paths, bounded reads, atomic,
       leak-free persistence)
  FL8: filesystem layout MCP contract (advertised inputSchema, audit target, destructive verdict,
       grant scope.paths confinement and canonical alias matching, nested-injection refusal)
  FL9: filesystem layout configuration contract (CLI wire suite U1..U12 for the T-01542
       validation contract + T-01544 store parse contract)
  FL10: filesystem layout automated lifecycle & edge cases (A1..A8: state machine, protection,
        fstab, probe, diff, corruption, audit)
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
        res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=180)
    except subprocess.TimeoutExpired:
        print(f"[-] {criterion} timed out after 180s", file=sys.stderr)
        return False
    except Exception as e:
        print(f"[-] {criterion} execution error: {e}", file=sys.stderr)
        return False

    if res.returncode != 0:
        print(f"[-] {criterion} cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print(f"[+] {criterion} {description}")
    return True


def _run_python_script(script_path: Path, criterion: str, description: str) -> bool:
    try:
        res = subprocess.run(
            [sys.executable, str(script_path)],
            capture_output=True,
            text=True,
            cwd=str(ROOT),
            timeout=180,
        )
    except subprocess.TimeoutExpired:
        print(f"[-] {criterion} timed out after 180s", file=sys.stderr)
        return False
    except Exception as e:
        print(f"[-] {criterion} execution error: {e}", file=sys.stderr)
        return False

    if res.returncode != 0:
        print(f"[-] {criterion} python test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print(f"[+] {criterion} {description}")
    return True


def test_fl1_data_model() -> bool:
    return _run_cargo_test(
        ["-p", "aiosh-core", "--test", "test_fs_layout_data_model"],
        "FL1",
        "filesystem layout data model integrity & invariants (FL1..FL5)",
    )


def test_fl2_core_service() -> bool:
    return _run_cargo_test(
        ["-p", "aiosh-core", "--test", "test_fs_layout_service"],
        "FL2",
        "filesystem layout core service (store, probe, diff, fstab, persistence)",
    )


def test_fl3_cli_surface() -> bool:
    return _run_python_script(
        ROOT / "code" / "aiosh-cli" / "tests" / "test_fs_layout_cli_smoke.py",
        "FL3",
        "filesystem layout CLI surface smoke & boundaries (cmd_fs_layout)",
    )


def test_fl4_cli_audit_security() -> bool:
    return _run_python_script(
        ROOT / "code" / "aiosh-cli" / "tests" / "test_fs_layout_audit_security.py",
        "FL4",
        "filesystem layout CLI audit emission & escape-injection security proof",
    )


def test_fl5_in_tree_unit_tests() -> bool:
    cli_ok = _run_cargo_test(
        ["-p", "aiosh-cli", "--bin", "aiosh", "test_cmd_fs_layout_flow"],
        "FL5",
        "filesystem layout CLI in-tree unit test",
    )
    if not cli_ok:
        return False
    return _run_cargo_test(
        ["-p", "aiosh-mcp", "--bin", "aiosh-mcp", "test_mcp_fs_layout_tools"],
        "FL5",
        "filesystem layout MCP in-tree unit test",
    )


def test_fl6_cross_surface_integration() -> bool:
    return _run_python_script(
        ROOT / "code" / "aiosh-mcp" / "tests" / "test_fs_layout_mcp_smoke.py",
        "FL6",
        "filesystem layout cross-surface CLI <-> MCP integration parity",
    )


def test_fl7_cli_hardening() -> bool:
    return _run_python_script(
        ROOT / "code" / "aiosh-cli" / "tests" / "test_fs_layout_hardening.py",
        "FL7",
        "filesystem layout CLI hardening (non-regular paths, bounded reads, atomic persistence)",
    )


def test_fl8_mcp_contract() -> bool:
    return _run_python_script(
        ROOT / "code" / "aiosh-mcp" / "tests" / "test_fs_layout_mcp_contract.py",
        "FL8",
        "filesystem layout MCP contract (advertised schema, audit target, destructive verdict, "
        "grant scope.paths confinement and canonical alias matching, nested-injection refusal)",
    )


def test_fl9_configuration_contract() -> bool:
    return _run_python_script(
        ROOT / "code" / "aiosh-cli" / "tests" / "test_fs_layout_config_validation.py",
        "FL9",
        "filesystem layout configuration contract (CLI wire suite: T-01542 validation rules "
        "E-1..E-7, ordering, fail-closed refusals, T-01544 store parse contract)",
    )


def test_fl10_automated_cases() -> bool:
    return _run_python_script(
        ROOT / "code" / "aiosh-cli" / "tests" / "test_fs_layout_automated_cases.py",
        "FL10",
        "filesystem layout automated lifecycle & edge cases (A1..A8: state machine, "
        "protection, fstab, probe, diff, corruption, audit)",
    )


def main() -> int:
    suites = [
        ("FL1", test_fl1_data_model),
        ("FL2", test_fl2_core_service),
        ("FL3", test_fl3_cli_surface),
        ("FL4", test_fl4_cli_audit_security),
        ("FL5", test_fl5_in_tree_unit_tests),
        ("FL6", test_fl6_cross_surface_integration),
        ("FL7", test_fl7_cli_hardening),
        ("FL8", test_fl8_mcp_contract),
        ("FL9", test_fl9_configuration_contract),
        ("FL10", test_fl10_automated_cases),
    ]

    failed = []
    for criterion, runner in suites:
        if not runner():
            failed.append(criterion)

    print()
    if failed:
        print(f"FAIL: fs_layout_suites failed criteria: {', '.join(failed)}", file=sys.stderr)
        return 1

    print("PASS: fs_layout_suites criteria (FL1..FL10)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
