#!/usr/bin/env python3
"""
Test runner for Secrets & Access Hygiene test suites (K1..K7).
"""

import sys
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

def test_k1_data_model_integrity():
    # Verify cargo test on secrets data model
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "secrets::tests"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K1 data model cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K1 data model integrity")
    return True

def test_k2_private_key_scanner():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "secrets_service::tests::test_scan_file_private_key"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K2 private key scanner failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K2 private key scanner")
    return True

def test_k3_api_token_scanner():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "secrets_service::tests::test_scan_file_aws_key_and_ghp"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K3 API token scanner failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K3 API token scanner")
    return True

def test_k4_config_password_scanner():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "secrets_service::tests::test_scan_file_password_in_config"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K4 config password scanner failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K4 config & env credentials scanner")
    return True

def test_k5_cli_surface():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--bin", "aiosh", "task_cli_tests::test_cmd_secrets_scan_and_check"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K5 CLI surface tests failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K5 CLI surface commands & options")
    return True

def test_k6_mcp_surface():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--bin", "aiosh-mcp", "tests::test_mcp_secrets_tools_execution"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K6 MCP surface tests failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K6 MCP tool schemas & execution")
    return True

def test_k7_config_suite():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "secrets_config::tests"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K7 SecretsConfig tests failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K7 SecretsConfig schema, validation & roundtrip")
    return True


def test_k8_observability_suite():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "secrets::tests::test_secret_scan_report_observability"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K8 observability tests failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K8 observability & scan telemetry")
    return True


def test_k9_recovery_and_validation():
    cmd = ["cargo", "test", "--manifest-path", str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"), "--lib", "secrets::tests::test_validate_secret_report_invalid"]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] K9 recovery and validation tests failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] K9 recovery & report validation invariants")
    return True


def main():
    checks = [
        test_k1_data_model_integrity,
        test_k2_private_key_scanner,
        test_k3_api_token_scanner,
        test_k4_config_password_scanner,
        test_k5_cli_surface,
        test_k6_mcp_surface,
        test_k7_config_suite,
        test_k8_observability_suite,
        test_k9_recovery_and_validation,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: secrets_suites criteria (K1..K9)")
        return 0
    else:
        print("\nFAIL: secrets_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())



