#!/usr/bin/env python3
"""Automated compound lifecycle and stress test cases for Kernel Module Management (T-01654).

Enforces:
- Full compound lifecycle sequences via CLI.
- Boundary values: max length module names (64 chars), long options (1024 bytes).
- Corruption recovery: failure handling on truncated and malformed JSON stores.
- Concurrent store isolation: multi-tenant store separation under distinct directories.

Run standalone:
    python code/aiosh-cli/tests/test_kernel_module_automated_cases.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def get_binary_path() -> str:
    candidates = [
        ROOT / "code/aiosh-rust/target/debug/aiosh.exe",
        ROOT / "code/aiosh-rust/target/debug/aiosh",
        ROOT / "target/debug/aiosh.exe",
        ROOT / "target/debug/aiosh",
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    return "aiosh"


def run_aiosh(*args: str) -> subprocess.CompletedProcess:
    res = subprocess.run([get_binary_path(), *args], capture_output=True, text=True, timeout=60)
    return res


def test_lifecycle_compound_workflow() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_path = str(tmp_path / "lifecycle.json")

        # 1. Blacklist
        res = run_aiosh("mod", "blacklist", "floppy", "--store", store_path, "--json")
        assert res.returncode == 0

        # 2. Options
        res = run_aiosh("mod", "options", "e1000e", "InterruptThrottleRate=1", "--store", store_path, "--json")
        assert res.returncode == 0

        # 3. Autoload
        res = run_aiosh("mod", "autoload", "kvm_intel", "--store", store_path, "--json")
        assert res.returncode == 0

        # 4. Preset apply
        res = run_aiosh("mod", "preset", "apply", "cis_hardened_baseline", "--store", store_path, "--json")
        assert res.returncode == 0

        # 5. List and verify
        res = run_aiosh("mod", "list", "--store", store_path, "--json")
        assert res.returncode == 0
        data = json.loads(res.stdout)["data"]
        assert "kvm_intel" in data["autoload_modules"]
        assert any(
            isinstance(r, dict) and (r.get("module") == "floppy" or r.get("blacklist", {}).get("module") == "floppy")
            for r in data["rules"]
        )

        # 6. Unautoload
        res = run_aiosh("mod", "unautoload", "kvm_intel", "--store", store_path, "--json")
        assert res.returncode == 0

        # 7. Unblacklist
        res = run_aiosh("mod", "unblacklist", "floppy", "--store", store_path, "--json")
        assert res.returncode == 0

    print("PASS: test_lifecycle_compound_workflow")


def test_boundary_value_cases() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        store_path = str(tmp_path / "boundary.json")

        # 1. Module name exactly 64 characters (maximum allowed)
        max_mod_name = "m" * 64
        res = run_aiosh("mod", "blacklist", max_mod_name, "--store", store_path, "--json")
        assert res.returncode == 0, f"failed for 64-char module name: {res.stdout}"

        # 2. Module name 65 characters (exceeds limit)
        oversized_mod_name = "m" * 65
        res = run_aiosh("mod", "blacklist", oversized_mod_name, "--store", store_path, "--json")
        assert res.returncode == 1, f"expected failure for 65-char module name: {res.stdout}"

        # 3. Parameter value exactly 1024 bytes (maximum allowed)
        long_val = "v" * 1024
        res = run_aiosh("mod", "options", "my_mod", f"param={long_val}", "--store", store_path, "--json")
        assert res.returncode == 0, f"failed for 1024-byte parameter: {res.stdout}"

        # 4. Parameter value 1025 bytes (exceeds limit)
        toolong_val = "v" * 1025
        res = run_aiosh("mod", "options", "my_mod", f"param={toolong_val}", "--store", store_path, "--json")
        assert res.returncode == 1, f"expected failure for 1025-byte parameter: {res.stdout}"

    print("PASS: test_boundary_value_cases")


def test_corrupt_store_cli_behavior() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        corrupt_store = str(tmp_path / "broken.json")
        with open(corrupt_store, "w") as f:
            f.write("INVALID JSON CONTENT {")

        res = run_aiosh("mod", "list", "--store", corrupt_store, "--json")
        assert res.returncode == 1, f"expected exit code 1 for corrupt store, got {res.returncode}"
        err = json.loads(res.stdout)["error"]
        assert err["code"] == "LOAD_STORE_FAILED"

    print("PASS: test_corrupt_store_cli_behavior")


def test_concurrent_store_isolation() -> None:
    with tempfile.TemporaryDirectory() as tmpdir1, tempfile.TemporaryDirectory() as tmpdir2:
        store1 = str(Path(tmpdir1) / "store.json")
        store2 = str(Path(tmpdir2) / "store.json")

        run_aiosh("mod", "blacklist", "mod_one", "--store", store1, "--json")
        run_aiosh("mod", "blacklist", "mod_two", "--store", store2, "--json")

        res1 = run_aiosh("mod", "list", "--store", store1, "--json")
        res2 = run_aiosh("mod", "list", "--store", store2, "--json")

        data1 = json.loads(res1.stdout)["data"]
        data2 = json.loads(res2.stdout)["data"]

        rules1 = [r.get("module") or r.get("blacklist", {}).get("module") for r in data1["rules"]]
        rules2 = [r.get("module") or r.get("blacklist", {}).get("module") for r in data2["rules"]]

        assert "mod_one" in rules1 and "mod_two" not in rules1
        assert "mod_two" in rules2 and "mod_one" not in rules2

    print("PASS: test_concurrent_store_isolation")


def main() -> None:
    print(f"Using binary: {get_binary_path()}")
    test_lifecycle_compound_workflow()
    test_boundary_value_cases()
    test_corrupt_store_cli_behavior()
    test_concurrent_store_isolation()
    print("ALL AUTOMATED LIFECYCLE TESTS PASSED.")


if __name__ == "__main__":
    main()
