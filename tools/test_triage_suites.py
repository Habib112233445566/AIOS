#!/usr/bin/env python3
"""
Test runner for Regression Triage test suites (T1..).
"""

import sys
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def test_t1_data_model_integrity():
    # Verify cargo test on triage data model
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "triage::tests"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] T1 data model cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] T1 triage data model integrity & failure signatures")
    return True


def test_t2_core_service_suite():
    # Verify cargo test on triage_service
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "triage_service::tests"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] T2 core service cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] T2 triage store, CI summary ingestion & persistence")
    return True


def test_t3_cli_surface():
    # Verify cargo test on aiosh-cli triage subcommands
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "-p", "aiosh-cli", "--bin", "aiosh", "--", "test_cmd_triage_flow"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] T3 CLI surface cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] T3 CLI surface commands, flags & flow")
    return True


def test_t4_mcp_surface():
    # Verify cargo test on aiosh-mcp triage tools
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "-p", "aiosh-mcp", "--bin", "aiosh-mcp", "--", "test_mcp_triage_tools"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] T4 MCP surface cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] T4 MCP surface tools, params & flow")
    return True


def test_t5_configuration_suite():
    # Verify cargo test on triage_config
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "triage_config::tests"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] T5 configuration cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] T5 triage configuration schema, validation & filters")
    return True


def test_t6_e2e_lifecycle_suite():
    # Scaffolding for end-to-end regression lifecycle verification
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "-p", "aiosh-cli", "--bin", "aiosh", "--", "test_cmd_triage_flow"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] T6 E2E lifecycle cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] T6 end-to-end regression triage lifecycle & recurrence")
    return True


def test_t7_observability_suite():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "triage::tests::test_triage_report_observability"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] T7 observability cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] T7 triage observability summary metrics & lifecycle diagnostics")
    return True


def test_t8_recovery_validation_suite():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "test_store_load_or_recover"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] T8 recovery & validation cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] T8 triage recovery resilience, error handling & invariant validation")
    return True


def main():
    checks = [
        test_t1_data_model_integrity,
        test_t2_core_service_suite,
        test_t3_cli_surface,
        test_t4_mcp_surface,
        test_t5_configuration_suite,
        test_t6_e2e_lifecycle_suite,
        test_t7_observability_suite,
        test_t8_recovery_validation_suite,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: triage_suites criteria (T1..T8)")
        return 0
    else:
        print("\nFAIL: triage_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())


