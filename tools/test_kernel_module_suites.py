#!/usr/bin/env python3
"""Unified Test Suite Orchestrator for Kernel Module Management (T-01654).

Executes all 8 Kernel Module test batteries in sequence:
- KM1: Core Data Model Unit Tests
- KM2: Core Service Unit Tests
- KM3: Configuration Unit Tests
- KM4: Automated In-Tree Integration Tests
- KM5: Operator CLI Smoke Suite
- KM6: Agent MCP Smoke Suite
- KM7: Configuration CLI Smoke Suite
- KM8: Automated Compound Lifecycle Suite
"""

from __future__ import annotations

import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

SUITES = [
    ("KM1: Core Data Model Unit Tests", ["cargo", "test", "-p", "aiosh-core", "--test", "test_kernel_module_data_model"], ROOT / "code" / "aiosh-rust"),
    ("KM2: Core Service Unit Tests", ["cargo", "test", "-p", "aiosh-core", "--test", "test_kernel_module_service"], ROOT / "code" / "aiosh-rust"),
    ("KM3: Configuration Unit Tests", ["cargo", "test", "-p", "aiosh-core", "--test", "test_kernel_module_config"], ROOT / "code" / "aiosh-rust"),
    ("KM4: Automated In-Tree Integration Tests", ["cargo", "test", "-p", "aiosh-core", "--test", "test_kernel_module_automated"], ROOT / "code" / "aiosh-rust"),
    ("KM5: Operator CLI Smoke Suite", [sys.executable, str(ROOT / "code" / "aiosh-cli" / "tests" / "test_kernel_module_cli_smoke.py")], ROOT),
    ("KM6: Agent MCP Smoke Suite", [sys.executable, str(ROOT / "code" / "aiosh-mcp" / "tests" / "test_kernel_module_mcp_smoke.py")], ROOT),
    ("KM7: Configuration CLI Smoke Suite", [sys.executable, str(ROOT / "code" / "aiosh-cli" / "tests" / "test_kernel_module_config_smoke.py")], ROOT),
    ("KM8: Automated Compound Lifecycle Suite", [sys.executable, str(ROOT / "code" / "aiosh-cli" / "tests" / "test_kernel_module_automated_cases.py")], ROOT),
]


def run_suite(name: str, cmd: list[str], cwd: Path) -> bool:
    print(f"\n==================================================")
    print(f"RUNNING: {name}")
    print(f"COMMAND: {' '.join(cmd)}")
    print(f"==================================================")
    start = time.time()
    res = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=180)
    elapsed = time.time() - start

    if res.stdout:
        print(res.stdout.strip())
    if res.stderr:
        print(res.stderr.strip(), file=sys.stderr)

    if res.returncode == 0:
        print(f"--> PASS: {name} ({elapsed:.2f}s)")
        return True
    else:
        print(f"--> FAIL: {name} (exit code {res.returncode}, {elapsed:.2f}s)")
        return False


def main() -> None:
    print("AIOS Kernel Module Management — Aggregate Test Orchestrator")
    passed = 0
    failed = 0

    for name, cmd, cwd in SUITES:
        if run_suite(name, cmd, cwd):
            passed += 1
        else:
            failed += 1
            break

    print("\n==================================================")
    print(f"SUMMARY: {passed} passed, {failed} failed (total {len(SUITES)})")
    print("==================================================")

    if failed > 0:
        sys.exit(1)
    else:
        print("ALL KERNEL MODULE MANAGEMENT SUITES PASSED.")
        sys.exit(0)


if __name__ == "__main__":
    main()
