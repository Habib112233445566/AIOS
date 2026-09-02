#!/usr/bin/env python3
"""Automated test suite runner for Agent Handoff Protocol (T-00911..T-01000).

Criteria:
  H1: Data model integrity, signature determinism & serde roundtrip
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def test_h1_data_model_integrity():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "handoff::tests"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] H1 data model cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] H1 handoff data model integrity & signature determinism")
    return True


def test_h2_core_service_suite():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "handoff_service::tests"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] H2 core service cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] H2 handoff store lifecycle, transitions & persistence")
    return True


def test_h3_cli_surface():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--bin", "aiosh", "test_cmd_handoff_flow"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] H3 CLI surface cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] H3 handoff CLI surface subcommands & flow")
    return True


def test_h4_mcp_surface():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--bin", "aiosh-mcp", "test_mcp_handoff_tools"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] H4 MCP surface cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] H4 handoff MCP surface tools & flow")
    return True


def test_h5_configuration():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "handoff_config::tests"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] H5 configuration cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] H5 handoff configuration schema, validation & roundtrip")
    return True


def test_h6_automated_suite():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "test_handoff_automated_edge_cases"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] H6 automated suite cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] H6 handoff automated edge cases, state matrix & batch fuzzing")
    return True


def test_h7_security_policy():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "test_handoff_authorization_matrix"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] H7 security policy cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] H7 handoff security policy & actor authorization matrix")
    return True


def test_h8_observability():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "test_handoff_report_validation_and_serde"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] H8 observability cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] H8 handoff observability metrics, status aggregation & reports")
    return True


def main():
    checks = [
        test_h1_data_model_integrity,
        test_h2_core_service_suite,
        test_h3_cli_surface,
        test_h4_mcp_surface,
        test_h5_configuration,
        test_h6_automated_suite,
        test_h7_security_policy,
        test_h8_observability,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: handoff_suites criteria (H1..H8)")
        return 0
    else:
        print("\nFAIL: handoff_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
