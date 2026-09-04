#!/usr/bin/env python3
"""Standalone test runner for Phase 1 Package Management subsystem.

Criteria:
  PM1: package data model integrity & invariants (PM1..PM5)
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


def main():
    checks = [
        test_pm1_data_model_integrity,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: package_suites criteria (PM1)")
        return 0
    else:
        print("\nFAIL: package_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
