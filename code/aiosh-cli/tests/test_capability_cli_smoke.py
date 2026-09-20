#!/usr/bin/env python3
"""CLI Smoke & Invariant Test for Capability Model (T-02021..T-02026)."""

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


def test_capability_help():
    res = run_aiosh("capability", "--help")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    assert "aiosh capability" in res.stdout
    assert "list" in res.stdout
    assert "show" in res.stdout
    assert "issue" in res.stdout
    assert "attenuate" in res.stdout
    assert "revoke" in res.stdout
    assert "check" in res.stdout
    assert "prune" in res.stdout
    print("PASS: aiosh capability --help")


def test_capability_unknown_subcommand():
    res = run_aiosh("capability", "unknown_cmd")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"
    print("PASS: aiosh capability unknown_cmd returns 2")


def test_capability_path_hygiene():
    with tempfile.TemporaryDirectory() as tmpdir:
        bad_ext = str(Path(tmpdir) / "caps.txt")
        res = run_aiosh("capability", "list", "--store", bad_ext)
        assert res.returncode == 2, f"Expected 2 for invalid extension, got {res.returncode}"

        traversal = str(Path(tmpdir) / "sub/../caps.json")
        res2 = run_aiosh("capability", "list", "--store", traversal)
        assert res2.returncode == 2, f"Expected 2 for traversal, got {res2.returncode}"
    print("PASS: aiosh capability path hygiene enforcement")


def test_capability_lifecycle():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "capabilities.json")

        # 1. Initial list empty
        res = run_aiosh("capability", "list", "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert len(data["data"]) == 0

        # 2. Issue root with invalid issuer -> fails with 1
        res = run_aiosh(
            "capability", "issue",
            "--issuer", "unauthorized_agent",
            "--subject", "agent:admin",
            "--scope-type", "filesystem",
            "--scope-target", "/var/data",
            "--rights", "read,write,delegate",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1

        # 3. Issue root with valid issuer 'kernel' -> succeeds
        res = run_aiosh(
            "capability", "issue",
            "--issuer", "kernel",
            "--subject", "agent:admin",
            "--scope-type", "filesystem",
            "--scope-target", "/var/data",
            "--rights", "read,write,delegate",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        root_id = data["data"]["id"]
        assert root_id.startswith("cap_")

        # 4. Show root capability
        res = run_aiosh("capability", "show", root_id, "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["data"]["id"] == root_id
        assert data["data"]["subject"] == "agent:admin"

        # 5. Attenuate child capability
        res = run_aiosh(
            "capability", "attenuate",
            "--parent", root_id,
            "--subject", "agent:worker",
            "--rights", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0
        data = parse_json_output(res)
        child_id = data["data"]["id"]
        assert data["data"]["parent_id"] == root_id
        assert data["data"]["subject"] == "agent:worker"

        # 6. Check access: granted for child read
        res = run_aiosh(
            "capability", "check",
            "--subject", "agent:worker",
            "--scope-type", "filesystem",
            "--scope-target", "/var/data",
            "--right", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["data"]["granted"] is True

        # Check access: denied for child write
        res = run_aiosh(
            "capability", "check",
            "--subject", "agent:worker",
            "--scope-type", "filesystem",
            "--scope-target", "/var/data",
            "--right", "write",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1
        data = parse_json_output(res)
        assert data["data"]["granted"] is False

        # 7. Revoke parent -> cascades to child
        res = run_aiosh("capability", "revoke", root_id, "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["data"]["revoked_count"] == 2
        assert root_id in data["data"]["revoked_ids"]
        assert child_id in data["data"]["revoked_ids"]

        # Check child access now denied
        res = run_aiosh(
            "capability", "check",
            "--subject", "agent:worker",
            "--scope-type", "filesystem",
            "--scope-target", "/var/data",
            "--right", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1

        # 8. Prune
        res = run_aiosh("capability", "prune", "--store", store, "--json")
        assert res.returncode == 0
    print("PASS: aiosh capability full lifecycle (issue, show, attenuate, check, revoke, prune)")


def main():
    print("=== aiosh capability CLI Smoke Tests ===")
    test_capability_help()
    test_capability_unknown_subcommand()
    test_capability_path_hygiene()
    test_capability_lifecycle()
    print("ALL CAPABILITY CLI SMOKE TESTS PASSED.")


if __name__ == "__main__":
    main()
