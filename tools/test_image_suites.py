#!/usr/bin/env python3
"""
test_image_suites.py - AIOS Linux Base Image Build Test Suite Runner
Validates data model, build services, CLI, MCP tools, and reproducibility invariants.
"""

import subprocess
import sys


def _run_cargo_test(test_args, criterion_id, description):
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        "code/aiosh-rust/Cargo.toml",
    ] + test_args

    try:
        res = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=120,
        )
    except subprocess.TimeoutExpired:
        print(f"[-] {criterion_id} {description} (TIMEOUT)", file=sys.stderr)
        return False
    except Exception as exc:
        print(f"[-] {criterion_id} {description} (ERROR: {exc})", file=sys.stderr)
        return False

    if res.returncode == 0:
        print(f"[+] {criterion_id} {description}")
        return True
    else:
        print(f"[-] {criterion_id} {description} (FAILED)", file=sys.stderr)
        print(res.stdout, file=sys.stderr)
        print(res.stderr, file=sys.stderr)
        return False


def test_b1_data_model_integrity():
    return _run_cargo_test(
        ["--lib", "base_image::tests"],
        "B1",
        "base image data model integrity & invariant validation",
    )


def test_b2_core_service_suite():
    return _run_cargo_test(
        ["--lib", "base_image_service::tests"],
        "B2",
        "base image store registry, persistence & build plan synthesis",
    )


def test_b3_cli_surface():
    return _run_cargo_test(
        ["--bin", "aiosh", "test_cmd_image_flow"],
        "B3",
        "base image CLI surface commands & options (list/show/plan/filter)",
    )


def test_b4_mcp_surface():
    return _run_cargo_test(
        ["--bin", "aiosh-mcp", "test_mcp_image_tools"],
        "B4",
        "base image MCP surface tools (list/get/plan)",
    )


def test_b5_configuration_invariants():
    return _run_cargo_test(
        ["--lib", "base_image_config::tests"],
        "B5",
        "base image configuration invariants & precedence (CF1..CF6)",
    )


def test_b6_automated_integration_suite():
    return _run_cargo_test(
        ["--test", "test_base_image_automated"],
        "B6",
        "base image automated integration test suite (T1..T7)",
    )


def test_b7_security_policy_suite():
    return _run_cargo_test(
        ["--test", "test_base_image_policy"],
        "B7",
        "base image security policy enforcement & invariants (P1..P7)",
    )


def main():
    checks = [
        test_b1_data_model_integrity,
        test_b2_core_service_suite,
        test_b3_cli_surface,
        test_b4_mcp_surface,
        test_b5_configuration_invariants,
        test_b6_automated_integration_suite,
        test_b7_security_policy_suite,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if all_ok:
        print("\nPASS: image_suites criteria (B1..B7)")
        return 0
    else:
        print("\nFAIL: image_suites criteria", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
