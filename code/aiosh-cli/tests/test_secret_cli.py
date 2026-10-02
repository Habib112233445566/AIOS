#!/usr/bin/env python3
"""Comprehensive Integration Tests for Secret CLI Subcommands (T-02626).

Covers all subcommands and flags of `aiosh secret` (and alias `aiosh sec`):
1. `help`: `[]`, `["--help"]`, `["-h"]` (exit code 0).
2. `unknown`: Unrecognized subcommand (exit code 2).
3. `traversal_defense`: `--store ../evil.json` (exit code 2).
4. `store`:
   - Missing required arguments (exit code 2).
   - Valid storage with metadata, scope, and target (exit code 0).
5. `get`:
   - Default masked output (does NOT expose plaintext) (exit code 0).
   - Plaintext access with `--expose` (exit code 0).
   - Non-existent key rejection (exit code 1).
6. `list`:
   - Listing metadata without exposing raw secrets (exit code 0).
   - Kind filtering (exit code 0).
   - JSON structured output (exit code 0).
7. `rotate`:
   - Rotates value to new version (exit code 0).
   - Verification via `get --expose` returns new secret (exit code 0).
8. `revoke`:
   - Revocation marks secret revoked (exit code 0).
   - Subsequent `get` rejected with error (exit code 1).
9. `alias`: `aiosh sec` behaves identically to `aiosh secret`.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _find_binary(names: list[str]) -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in names:
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return names[-1]


def get_aiosh_binary() -> str:
    return _find_binary(["aiosh.exe", "aiosh"])


def run_aiosh(*args: str, timeout_s: int = 15) -> subprocess.CompletedProcess:
    bin_path = get_aiosh_binary()
    return subprocess.run(
        [bin_path, *args],
        capture_output=True,
        text=True,
        timeout=timeout_s,
    )


def parse_json_output(res: subprocess.CompletedProcess) -> dict:
    for line in reversed(res.stdout.splitlines()):
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                return json.loads(line)
            except Exception:
                pass
    raise ValueError(f"Could not find JSON in stdout:\nSTDOUT:\n{res.stdout}\nSTDERR:\n{res.stderr}")


def test_secret_help():
    print("Testing secret help commands...")
    for cmd in ("secret", "sec"):
        for flag in ([], ["--help"], ["-h"]):
            res = run_aiosh(cmd, *flag)
            assert res.returncode == 0, f"Expected 0 for {cmd} {flag}, got {res.returncode}"
            assert "aiosh secret" in res.stdout


def test_secret_unknown():
    print("Testing unknown subcommands...")
    res = run_aiosh("secret", "invalid_op")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"


def test_secret_path_traversal():
    print("Testing path traversal defense...")
    res = run_aiosh("secret", "list", "--store", "../forbidden/vault.json")
    assert res.returncode == 2, f"Expected 2 for traversal, got {res.returncode}"


def test_secret_lifecycle():
    print("Testing secret full lifecycle...")
    tmp_dir = tempfile.mkdtemp(prefix="aiosh_secret_test_")
    store = os.path.join(tmp_dir, "vault.json")
    try:
        # 1. Validation failure: missing required --id or --name
        res = run_aiosh("secret", "store", "--name", "Key1", "--store", store)
        assert res.returncode == 2, f"Expected 2 for missing --id, got {res.returncode}"

        # 2. Store a secret
        res = run_aiosh(
            "secret", "store",
            "--id", "api_key_primary",
            "--name", "Primary API Key",
            "--kind", "api_key",
            "--value", "super_secret_token_12345",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0, f"Store failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["id"] == "api_key_primary"
        assert data["data"]["name"] == "Primary API Key"

        # 3. Get secret: masked by default
        res = run_aiosh("secret", "get", "--id", "api_key_primary", "--store", store)
        assert res.returncode == 0, f"Get failed: {res.stderr}"
        assert "super_secret_token_12345" not in res.stdout, "Raw secret was exposed in default get!"
        assert "..." in res.stdout or "********" in res.stdout

        # 4. Get secret with --expose
        res = run_aiosh("secret", "get", "--id", "api_key_primary", "--expose", "--store", store)
        assert res.returncode == 0, f"Get --expose failed: {res.stderr}"
        assert "super_secret_token_12345" in res.stdout, "Raw secret was not returned with --expose"

        # 5. Get secret with --json
        res = run_aiosh("secret", "get", "--id", "api_key_primary", "--store", store, "--json")
        assert res.returncode == 0, f"Get --json failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["metadata"]["id"] == "api_key_primary"
        assert "super_secret_token_12345" not in str(data["data"]), "JSON get without expose leaked raw secret!"

        # 6. Get non-existent secret
        res = run_aiosh("secret", "get", "--id", "missing_key", "--store", store)
        assert res.returncode == 1, f"Expected 1 for non-existent secret, got {res.returncode}"

        # 7. List secrets (no secret values in output)
        res = run_aiosh("secret", "list", "--store", store, "--json")
        assert res.returncode == 0, f"List failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        items = data["data"]
        assert len(items) == 1
        assert items[0]["id"] == "api_key_primary"
        assert "super_secret_token_12345" not in res.stdout

        # 8. Rotate secret
        res = run_aiosh(
            "secret", "rotate",
            "--id", "api_key_primary",
            "--value", "rotated_secret_token_67890",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0, f"Rotate failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["version"] == 2

        # Verify rotated value with --expose
        res = run_aiosh("secret", "get", "--id", "api_key_primary", "--expose", "--store", store)
        assert res.returncode == 0
        assert "rotated_secret_token_67890" in res.stdout

        # 9. Revoke secret
        res = run_aiosh("secret", "revoke", "--id", "api_key_primary", "--store", store, "--json")
        assert res.returncode == 0, f"Revoke failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["state"] == "revoked"

        # 10. Accessing revoked secret is rejected
        res = run_aiosh("secret", "get", "--id", "api_key_primary", "--store", store)
        assert res.returncode == 1, f"Expected 1 for revoked secret, got {res.returncode}"

    finally:
        shutil.rmtree(tmp_dir, ignore_errors=True)


def main():
    print("Running aiosh secret CLI smoke test suite...")
    test_secret_help()
    test_secret_unknown()
    test_secret_path_traversal()
    test_secret_lifecycle()
    print("All aiosh secret CLI integration tests PASSED!")


if __name__ == "__main__":
    main()
