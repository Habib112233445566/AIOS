#!/usr/bin/env python3
"""CLI Smoke & Unit Test for Evidence & Audit Trail (T-00535)."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

def get_binary_path():
    candidates = [
        Path("code/aiosh-rust/target/debug/aiosh.exe"),
        Path("code/aiosh-rust/target/debug/aiosh"),
        Path("target/debug/aiosh.exe"),
        Path("target/debug/aiosh"),
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

def test_evidence_hash_prose():
    res = run_aiosh("evidence", "hash", "docs/README.md")
    if res.returncode != 0:
        print(f"FAIL: aiosh evidence hash returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    assert "docs/README.md ->" in res.stdout
    print("PASS: aiosh evidence hash prose")

def test_evidence_hash_json():
    res = run_aiosh("evidence", "hash", "docs/README.md", "--json")
    if res.returncode != 0:
        print(f"FAIL: aiosh evidence hash --json returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    data = parse_json_output(res)
    assert data.get("ok") is True
    assert data.get("subcommand") == "evidence hash"
    assert "sha256" in data
    assert len(data["sha256"]) == 64
    print("PASS: aiosh evidence hash --json")

def test_evidence_hash_missing_file_error():
    res = run_aiosh("evidence", "hash", "non_existent_file_xyz.md", "--json")
    if res.returncode != 1:
        print(f"FAIL: expected returncode 1 for missing file, got {res.returncode}")
        sys.exit(1)
    data = parse_json_output(res)
    assert data.get("ok") is False
    assert "error" in data
    print("PASS: aiosh evidence hash missing file error")

def test_evidence_hash_missing_arg_error():
    res = run_aiosh("evidence", "hash")
    if res.returncode != 2:
        print(f"FAIL: expected returncode 2 for missing arg, got {res.returncode}")
        sys.exit(1)
    assert "usage:" in res.stderr
    print("PASS: aiosh evidence hash missing arg error")

def test_evidence_verify_default():
    res = run_aiosh("evidence", "verify", "--json")
    if res.returncode != 0:
        print(f"FAIL: aiosh evidence verify --json returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    data = parse_json_output(res)
    assert data.get("ok") is True
    assert data.get("subcommand") == "evidence verify"
    print("PASS: aiosh evidence verify --json")

def test_evidence_scan():
    res = run_aiosh("evidence", "scan", "--json")
    if res.returncode != 0:
        print(f"FAIL: aiosh evidence scan --json returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    data = parse_json_output(res)
    assert data.get("ok") is True
    assert data.get("subcommand") == "evidence scan"
    assert "data" in data
    assert len(data["data"]) > 0
    print("PASS: aiosh evidence scan --json")

def test_evidence_scan_filtered():
    res = run_aiosh("evidence", "scan", "--task", "501", "--json")
    if res.returncode != 0:
        print(f"FAIL: aiosh evidence scan --task 501 returned {res.returncode}")
        print(res.stderr)
        sys.exit(1)
    data = parse_json_output(res)
    assert data.get("ok") is True
    assert len(data.get("data", [])) >= 1
    for item in data["data"]:
        assert item["task_id"] == 501
    print("PASS: aiosh evidence scan filtered by task")

def test_evidence_unknown_subcommand():
    res = run_aiosh("evidence", "invalid_subcommand")
    if res.returncode != 2:
        print(f"FAIL: expected returncode 2 for unknown subcommand, got {res.returncode}")
        sys.exit(1)
    assert "usage:" in res.stderr
    print("PASS: aiosh evidence unknown subcommand error")

def main():
    test_evidence_hash_prose()
    test_evidence_hash_json()
    test_evidence_hash_missing_file_error()
    test_evidence_hash_missing_arg_error()
    test_evidence_verify_default()
    test_evidence_scan()
    test_evidence_scan_filtered()
    test_evidence_unknown_subcommand()
    print("All 8 evidence CLI unit and smoke tests passed successfully!")

if __name__ == "__main__":
    main()
