#!/usr/bin/env python3
"""Comprehensive Integration Tests for Privilege CLI Subcommands (T-02526).

Covers all subcommands and parameters of `aiosh privilege` (and `aiosh priv`):
1. `help`: Missing args, `--help`, `-h`.
2. `status`: Initializing User context and JSON output.
3. `elevate`:
   - Rejection without grant token (exit code 1).
   - Rejection of SystemKernel target (exit code 1).
   - Successful elevation with grant token (exit code 0).
4. `check`:
   - Verifying holding elevated capability (exit code 0).
   - Verifying lacking ungranted capability (exit code 1).
5. `drop`:
   - De-escalating tier and capability pruning (exit code 0).
6. `revoke`:
   - Reverting to baseline level (exit code 0).
7. `list`:
   - Listing registered actors.
8. `path_hygiene`: Rejection of traversal paths.
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


def test_privilege_help():
    print("Testing help commands...")
    for flag in ([], ["--help"], ["-h"]):
        res = run_aiosh("privilege", *flag)
        assert res.returncode == 0, f"Expected 0, got {res.returncode}"
        assert "aiosh privilege" in res.stdout


def test_privilege_unknown():
    print("Testing unknown subcommands...")
    res = run_aiosh("privilege", "nonexistent_subcmd")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"


def test_privilege_lifecycle():
    print("Testing privilege lifecycle...")
    tmp_dir = tempfile.mkdtemp(prefix="aiosh_priv_test_")
    store = os.path.join(tmp_dir, "privileges.json")
    try:
        # 1. Status initializes default User context
        res = run_aiosh("privilege", "status", "--actor", "test_actor_1", "--store", store, "--json")
        assert res.returncode == 0, f"Status failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["actor_id"] == "test_actor_1"
        assert data["data"]["active_level"] == "user"

        # 2. Elevate without grant fails with code 1
        res = run_aiosh("privilege", "elevate", "--to", "operator", "--actor", "test_actor_1", "--store", store)
        assert res.returncode == 1, f"Expected 1, got {res.returncode}"

        # 3. Elevate to SystemKernel is blocked with code 1
        res = run_aiosh("privilege", "elevate", "--to", "system_kernel", "--grant", "TOKEN-1", "--actor", "test_actor_1", "--store", store)
        assert res.returncode == 1, f"Expected 1, got {res.returncode}"

        # 4. Elevate with valid grant succeeds
        res = run_aiosh(
            "privilege", "elevate",
            "--to", "operator",
            "--grant", "GRANT-PEP-99",
            "--caps", "network_listen",
            "--actor", "test_actor_1",
            "--store", store,
            "--json"
        )
        assert res.returncode == 0, f"Elevation failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["active_level"] == "operator"
        assert data["data"]["is_elevation_active"] is True

        # 5. Check capability
        res = run_aiosh("privilege", "check", "--cap", "network_listen", "--actor", "test_actor_1", "--store", store)
        assert res.returncode == 0, f"Check failed: {res.stderr}"

        res = run_aiosh("privilege", "check", "--cap", "kernel_module_load", "--actor", "test_actor_1", "--store", store)
        assert res.returncode == 1, f"Expected 1, got {res.returncode}"

        # 6. Drop to user
        res = run_aiosh("privilege", "drop", "--to", "user", "--actor", "test_actor_1", "--store", store)
        assert res.returncode == 0, f"Drop failed: {res.stderr}"

        # Capability pruned after drop
        res = run_aiosh("privilege", "check", "--cap", "network_listen", "--actor", "test_actor_1", "--store", store)
        assert res.returncode == 1, f"Expected 1, got {res.returncode}"

        # 7. Revoke elevation
        res = run_aiosh("privilege", "revoke", "--actor", "test_actor_1", "--store", store)
        assert res.returncode == 0, f"Revoke failed: {res.stderr}"

        # 8. List actors
        res = run_aiosh("privilege", "list", "--store", store, "--json")
        assert res.returncode == 0, f"List failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        assert "test_actor_1" in data["data"]

        # 9. Path hygiene rejection
        res = run_aiosh("privilege", "status", "--store", "../bad.json")
        assert res.returncode == 2, f"Expected 2, got {res.returncode}"

    finally:
        shutil.rmtree(tmp_dir, ignore_errors=True)


def main():
    print("=== Running Privilege CLI Integration Tests ===")
    test_privilege_help()
    test_privilege_unknown()
    test_privilege_lifecycle()
    print("=== ALL PRIVILEGE CLI TESTS PASSED ===")


if __name__ == "__main__":
    main()
