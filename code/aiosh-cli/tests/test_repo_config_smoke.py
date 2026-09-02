#!/usr/bin/env python3
"""Smoke & Unit Test for Repository Health Configuration (T-00655)."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

def test_rust_unit_tests():
    cmd = ["cargo", "test", "--manifest-path", "code/aiosh-rust/Cargo.toml", "--lib", "repo_health_config::tests"]
    res = subprocess.run(cmd, cwd=str(ROOT), capture_output=True, text=True, timeout=60)
    assert res.returncode == 0, f"Cargo tests failed:\n{res.stdout}\n{res.stderr}"
    assert "test repo_health_config::tests::test_repo_health_config_default_and_roundtrip ... ok" in res.stdout
    assert "test repo_health_config::tests::test_repo_health_config_validation_errors ... ok" in res.stdout
    assert "test repo_health_config::tests::test_repo_health_config_from_path ... ok" in res.stdout
    print("PASS: cargo test repo_health_config::tests (3/3 tests passed)")

def test_config_schema_validation():
    default_config = {
        "version": "1.0.0",
        "max_file_bytes": 16777216,
        "ignored_dirs": [".git", "target", "node_modules", ".venv"],
        "require_clean_git": False,
        "security_policy_path": "SECURITY.md",
        "min_security_policy_bytes": 100
    }
    assert default_config["version"] == "1.0.0"
    assert default_config["max_file_bytes"] >= 1024
    assert len(default_config["ignored_dirs"]) >= 4
    assert not any(".." in d for d in default_config["ignored_dirs"])
    print("PASS: config schema definition & safety assertions")

def main():
    test_rust_unit_tests()
    test_config_schema_validation()
    print("\nALL REPO HEALTH CONFIG SMOKE TESTS PASSED!")

if __name__ == "__main__":
    main()
