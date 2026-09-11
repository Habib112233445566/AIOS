#!/usr/bin/env python3
"""Standalone test runner for Phase 1 User Session Bootstrap subsystem.

Criteria:
  SB1: session data model integrity & invariants (SB1..SB5)
  SB2: session CLI surface commands & options (validate, help, errors)
  SB3: session MCP tool surface (aios.session.validate)
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


def _run_python_script(script_path: Path, criterion: str, description: str) -> bool:
    try:
        res = subprocess.run(
            [sys.executable, str(script_path)],
            capture_output=True,
            text=True,
            cwd=str(ROOT),
            timeout=120,
        )
    except subprocess.TimeoutExpired:
        print(f"[-] {criterion} timed out after 120s", file=sys.stderr)
        return False
    except Exception as e:
        print(f"[-] {criterion} execution error: {e}", file=sys.stderr)
        return False

    if res.returncode != 0:
        print(f"[-] {criterion} python test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print(f"[+] {criterion} {description}")
    return True


def test_sb1_data_model_integrity():
    return _run_cargo_test(
        ["--test", "test_session_data_model"],
        "SB1",
        "session data model integrity & invariants (SB1..SB5)",
    )


def test_sb2_cli_surface_commands():
    cli_smoke = ROOT / "code" / "aiosh-cli" / "tests" / "test_session_cli_smoke.py"
    return _run_python_script(
        cli_smoke,
        "SB2",
        "session CLI surface commands & options (validate, help, errors)",
    )


def test_sb3_mcp_surface_tools():
    cargo_ok = _run_cargo_test(
        ["-p", "aiosh-mcp", "--bin", "aiosh-mcp", "test_mcp_session_validate_tools"],
        "SB3",
        "session MCP in-tree unit test suite",
    )
    if not cargo_ok:
        return False
    mcp_smoke = ROOT / "code" / "aiosh-mcp" / "tests" / "test_session_mcp_smoke.py"
    return _run_python_script(
        mcp_smoke,
        "SB3",
        "session MCP tool surface JSON-RPC smoke test (aios.session.*)",
    )


def test_sb4_core_service_lifecycle():
    return _run_cargo_test(
        ["--test", "test_session_service"],
        "SB4",
        "session core service lifecycle, seat arbitration & invariants (CS1..CS5)",
    )


def test_sb5_configuration():
    return _run_cargo_test(
        ["--test", "test_session_config"],
        "SB5",
        "session configuration resolution, invariants & precedence (SC1..SC7)",
    )


def main() -> int:
    suites = [
        ("SB1", test_sb1_data_model_integrity),
        ("SB2", test_sb2_cli_surface_commands),
        ("SB3", test_sb3_mcp_surface_tools),
        ("SB4", test_sb4_core_service_lifecycle),
        ("SB5", test_sb5_configuration),
    ]

    failed = []
    for criterion, runner in suites:
        if not runner():
            failed.append(criterion)

    print()
    if failed:
        print(f"FAIL: session_suites failed criteria: {', '.join(failed)}", file=sys.stderr)
        return 1

    print("PASS: session_suites criteria (SB1..SB5)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
