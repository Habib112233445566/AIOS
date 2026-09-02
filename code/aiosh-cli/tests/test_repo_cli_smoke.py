#!/usr/bin/env python3
"""CLI Smoke & Unit Test for Repository Health (T-00635)."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

def get_binary_path():
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

def run_aiosh(*args):
    bin_path = get_binary_path()
    cmd = [bin_path, *args]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    return res

def parse_json_output(res):
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)

def test_repo_health_prose():
    res = run_aiosh("repo", "health")
    assert res.returncode in (0, 1), f"Unexpected returncode {res.returncode}"
    assert "Repository Health Assessment" in res.stdout
    assert "Overall Status" in res.stdout
    print("PASS: aiosh repo health prose output")

def test_repo_health_json():
    res = run_aiosh("repo", "health", "--json")
    assert res.returncode in (0, 1), f"Unexpected returncode {res.returncode}"
    data = parse_json_output(res)
    assert "data" in data
    report = data["data"]
    assert "overall_status" in report
    assert "total_checks" in report
    assert report["total_checks"] >= 3
    assert "checks" in report
    print("PASS: aiosh repo health --json output")

def test_repo_check_alias():
    res = run_aiosh("repo", "check", "--json")
    assert res.returncode in (0, 1), f"Unexpected returncode {res.returncode}"
    data = parse_json_output(res)
    assert "data" in data
    assert "overall_status" in data["data"]
    print("PASS: aiosh repo check alias")

def test_repo_custom_path():
    with tempfile.TemporaryDirectory() as td:
        sec_path = Path(td) / "SECURITY.md"
        sec_path.write_text("# Security Policy\n\nValid security policy for automated test.\nNo disclosure issues found.\n")
        
        res = run_aiosh("repo", "health", "--repo", td, "--json")
        assert res.returncode in (0, 1), f"Unexpected returncode {res.returncode}"
        data = parse_json_output(res)
        assert data["data"]["total_checks"] == 3
        print("PASS: aiosh repo health --repo custom path")

def test_repo_invalid_subcommand():
    res = run_aiosh("repo", "non_existent_command")
    assert res.returncode == 2, f"Expected returncode 2 for invalid subcommand, got {res.returncode}"
    print("PASS: aiosh repo invalid subcommand rejection")

def main():
    test_repo_health_prose()
    test_repo_health_json()
    test_repo_check_alias()
    test_repo_custom_path()
    test_repo_invalid_subcommand()
    print("\nALL REPO CLI SMOKE TESTS PASSED!")

if __name__ == "__main__":
    main()
