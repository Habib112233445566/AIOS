#!/usr/bin/env python3
"""CLI Smoke & Invariant Test for PEP Decision Engine (T-02121..T-02125)."""

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
    bin_path = get_binary_path()
    cmd = [bin_path, *args]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    return res


def parse_json_output(res: subprocess.CompletedProcess) -> dict | list:
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)


def test_pep_help():
    res = run_aiosh("pep", "--help")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    assert "aiosh pep" in res.stdout
    assert "evaluate" in res.stdout
    assert "rule-add" in res.stdout
    assert "rule-list" in res.stdout
    assert "rule-remove" in res.stdout
    assert "status" in res.stdout
    print("PASS: aiosh pep --help")


def test_pep_unknown_subcommand():
    res = run_aiosh("pep", "unknown_cmd")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"
    print("PASS: aiosh pep unknown_cmd returns 2")


def test_pep_path_hygiene():
    with tempfile.TemporaryDirectory() as tmpdir:
        bad_ext = str(Path(tmpdir) / "pep.txt")
        res = run_aiosh("pep", "status", "--store", bad_ext)
        assert res.returncode == 2, f"Expected 2 for invalid extension, got {res.returncode}"

        traversal = str(Path(tmpdir) / "sub/../pep.json")
        res2 = run_aiosh("pep", "status", "--store", traversal)
        assert res2.returncode == 2, f"Expected 2 for traversal, got {res2.returncode}"
    print("PASS: aiosh pep path hygiene enforcement")


def test_pep_lifecycle():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_policies.json")

        # 1. Initial status empty
        res = run_aiosh("pep", "status", "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["rules_count"] == 0

        # 2. Evaluate without rules -> default deny (exit code 1)
        res = run_aiosh(
            "pep", "evaluate",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/q1.pdf",
            "--action", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1, f"Expected 1 for deny, got {res.returncode}"
        data = parse_json_output(res)
        assert data["code"] == 1
        assert data["data"]["allowed"] is False
        assert data["data"]["effect"] == "deny"

        # 3. Add rule with invalid effect -> fails with 2
        res = run_aiosh(
            "pep", "rule-add",
            "--id", "r1",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/*",
            "--action", "read",
            "--effect", "maybe",
            "--store", store,
            "--json",
        )
        assert res.returncode == 2

        # 4. Add rule with valid effect -> succeeds with 0
        res = run_aiosh(
            "pep", "rule-add",
            "--id", "rule_allow_reports",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/*",
            "--action", "read",
            "--effect", "permit",
            "--desc", "allow reports read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["id"] == "rule_allow_reports"

        # 5. Evaluate matching request -> permit (exit code 0)
        res = run_aiosh(
            "pep", "evaluate",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/q1.pdf",
            "--action", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["allowed"] is True
        assert data["data"]["effect"] == "permit"
        assert data["data"]["matched_rule_id"] == "rule_allow_reports"

        # 6. Evaluate non-matching action -> deny (exit code 1)
        res = run_aiosh(
            "pep", "evaluate",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/q1.pdf",
            "--action", "write",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1
        data = parse_json_output(res)
        assert data["code"] == 1
        assert data["data"]["allowed"] is False

        # 7. List rules
        res = run_aiosh("pep", "rule-list", "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert len(data["data"]) == 1

        # 8. Remove rule
        res = run_aiosh("pep", "rule-remove", "rule_allow_reports", "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0

        # Remove again -> not found (code 1)
        res = run_aiosh("pep", "rule-remove", "rule_allow_reports", "--store", store, "--json")
        assert res.returncode == 1

        # 9. Evaluate after removal -> deny (code 1)
        res = run_aiosh(
            "pep", "evaluate",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/q1.pdf",
            "--action", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1

    print("PASS: aiosh pep lifecycle and evaluation")


if __name__ == "__main__":
    test_pep_help()
    test_pep_unknown_subcommand()
    test_pep_path_hygiene()
    test_pep_lifecycle()
    print("=== All PEP CLI tests passed ===")
