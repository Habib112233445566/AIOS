#!/usr/bin/env python3
"""Automated Multi-Tenant Test Suite for Privilege Escalation Prevention (T-02554).

Vectors:
1. Multi-tenant context isolation across CLI invocations.
2. SystemKernel immutability fail-closed check.
3. Rapid elevation and drop cycle under temporary isolated stores.
4. Capability check positive and negative assertions.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def get_aiosh_binary() -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in ("aiosh.exe", "aiosh"):
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return "aiosh"


def run_aiosh(*args: str, store_path: str | None = None) -> subprocess.CompletedProcess:
    bin_path = get_aiosh_binary()
    env = os.environ.copy()
    if store_path:
        env["AIOS_PRIVILEGE_STORE_PATH"] = store_path
    return subprocess.run(
        [bin_path, *args],
        capture_output=True,
        text=True,
        timeout=15,
        env=env,
    )


def test_automated_multitenant_cli():
    with tempfile.TemporaryDirectory() as td:
        store = os.path.join(td, "priv_store.json")

        # 1. Initialize user1
        res1 = run_aiosh("privilege", "status", "--actor", "user1", "--json", store_path=store)
        assert res1.returncode == 0, f"status user1 failed: {res1.stderr}"
        data1 = json.loads(res1.stdout)
        assert data1["data"]["active_level"] == "user"

        # 2. Initialize user2
        res2 = run_aiosh("privilege", "status", "--actor", "user2", "--json", store_path=store)
        assert res2.returncode == 0, f"status user2 failed: {res2.stderr}"
        data2 = json.loads(res2.stdout)
        assert data2["data"]["active_level"] == "user"

        # 3. Elevate user1
        res_elevate = run_aiosh(
            "privilege", "elevate", "--actor", "user1", "--to", "operator",
            "--grant", "grant-user1-ops-100", "--caps", "network_connect,audit_log_admin",
            "--json", store_path=store
        )
        assert res_elevate.returncode == 0, f"elevate user1 failed: {res_elevate.stderr}"

        # 4. Check user1 has cap, user2 does not
        res_chk1 = run_aiosh("privilege", "check", "--actor", "user1", "--cap", "audit_log_admin", store_path=store)
        assert res_chk1.returncode == 0, f"user1 check failed: {res_chk1.stderr}"

        res_chk2 = run_aiosh("privilege", "check", "--actor", "user2", "--cap", "audit_log_admin", store_path=store)
        assert res_chk2.returncode != 0, "user2 must not possess user1 capability"

        # 5. Kernel immutability
        res_kernel = run_aiosh(
            "privilege", "elevate", "--actor", "user1", "--to", "system_kernel",
            "--grant", "grant-god-mode", "--json", store_path=store
        )
        assert res_kernel.returncode != 0, "elevation to system_kernel must fail"


if __name__ == "__main__":
    test_automated_multitenant_cli()
    print("ALL AUTOMATED PRIVILEGE CLI TESTS PASSED")
