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


def test_d1_data_model_integrity():
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"),
        "--lib",
        "distro::tests::test_distro_profile_validation_and_defaults",
    ]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] D1 cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] D1 distro data model integrity & validation invariants")
    return True


def test_d2_core_service_suite():
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"),
        "--lib",
        "distro_service::tests::test_distro_store_lifecycle_and_evaluations",
    ]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] D2 cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] D2 distro store lifecycle, registry querying & persistence")
    return True


def test_d3_cli_surface():
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"),
        "--bin",
        "aiosh",
        "test_cmd_distro_flow",
    ]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] D3 cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] D3 distro CLI surface commands & options (list/show/evaluate/recommend)")
    return True


def test_d4_mcp_surface():
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"),
        "--bin",
        "aiosh-mcp",
        "test_mcp_distro_tools",
    ]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] D4 cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] D4 distro MCP tools dispatch & execution (list/show/evaluate/recommend)")
    return True


def main():
    checks = [
        test_d1_data_model_integrity,
        test_d2_core_service_suite,
        test_d3_cli_surface,
        test_d4_mcp_surface,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: distro_suites criteria (D1..D4)")
        return 0
    else:
        print("\nFAIL: distro_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
