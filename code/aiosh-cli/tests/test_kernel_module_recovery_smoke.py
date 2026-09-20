#!/usr/bin/env python3
"""Integration Smoke Test for Kernel Module Management Recovery & Validation (T-01696).

Validates CLI (`aiosh mod check [--auto-recover]`) and MCP (`aios.kernel_module.check`)
surfaces for health check evaluation, corruption detection, automated self-healing,
quarantine backup creation, and cross-surface parity.
"""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def get_cli_path() -> str:
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


def get_mcp_path() -> str:
    candidates = [
        ROOT / "code/aiosh-rust/target/debug/aiosh-mcp.exe",
        ROOT / "code/aiosh-rust/target/debug/aiosh-mcp",
        ROOT / "target/debug/aiosh-mcp.exe",
        ROOT / "target/debug/aiosh-mcp",
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    return "aiosh-mcp"


def run_cli(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run([get_cli_path(), *args], capture_output=True, text=True, timeout=60)


def test_cli_check_healthy():
    with tempfile.TemporaryDirectory() as tmp:
        store_path = os.path.join(tmp, "store.json")
        # Initialize default store
        init_store = {
            "config": {
                "id": "smoke-test",
                "description": "Healthy test store",
                "rules": [
                    {"type": "blacklist", "module": "cramfs"}
                ],
                "autoload_modules": ["overlay"],
                "created_at": "2026-09-20T00:00:00Z"
            }
        }
        with open(store_path, "w", encoding="utf-8") as f:
            json.dump(init_store, f)

        res = run_cli("mod", "check", "--store", store_path, "--json")
        assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
        data = json.loads(res.stdout)
        assert data["code"] == 0
        rep = data["data"]
        assert rep["healthy"] is True
        assert rep["total_rules"] == 1
        assert rep["valid_rules"] == 1
        assert rep["invalid_rules"] == 0
        assert rep["total_autoload"] == 1
        assert rep["valid_autoload"] == 1
        assert rep["invalid_autoload"] == 0
        assert rep["recovered"] is False
    print("PASS: test_cli_check_healthy")


def test_cli_check_corrupted_and_auto_recover():
    with tempfile.TemporaryDirectory() as tmp:
        store_path = os.path.join(tmp, "corrupt.json")
        broken_content = "{ invalid json content ... "
        with open(store_path, "w", encoding="utf-8") as f:
            f.write(broken_content)

        # 1. Check without auto-recover should report failure (exit 1)
        res = run_cli("mod", "check", "--store", store_path, "--json")
        assert res.returncode == 1, f"Expected 1, got {res.returncode}: {res.stdout}"
        data = json.loads(res.stdout)
        assert data["code"] == 1
        assert data["data"]["healthy"] is False

        # 2. Check with --auto-recover should repair the store (exit 0)
        res_rec = run_cli("mod", "check", "--store", store_path, "--auto-recover", "--json")
        assert res_rec.returncode == 0, f"Expected 0, got {res_rec.returncode}: {res_rec.stderr}"
        data_rec = json.loads(res_rec.stdout)
        assert data_rec["code"] == 0
        rep = data_rec["data"]
        assert rep["healthy"] is True
        assert rep["recovered"] is True
        assert rep["backup_path"] is not None
        assert os.path.exists(rep["backup_path"])

        # Check quarantine content matches broken original
        with open(rep["backup_path"], "r", encoding="utf-8") as f:
            backup_content = f.read()
        assert backup_content == broken_content
    print("PASS: test_cli_check_corrupted_and_auto_recover")


def test_cli_check_human_output():
    with tempfile.TemporaryDirectory() as tmp:
        store_path = os.path.join(tmp, "store.json")
        init_store = {
            "config": {
                "id": "human-test",
                "description": "Human format test",
                "rules": [],
                "autoload_modules": [],
                "created_at": "2026-09-20T00:00:00Z"
            }
        }
        with open(store_path, "w", encoding="utf-8") as f:
            json.dump(init_store, f)

        res = run_cli("mod", "check", "--store", store_path)
        assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
        assert "Kernel Module Store Integrity Report:" in res.stdout
        assert "Status: HEALTHY" in res.stdout
        assert "Rules: 0 total (0 valid, 0 invalid)" in res.stdout
    print("PASS: test_cli_check_human_output")


def test_mcp_check_tool():
    p = subprocess.Popen(
        [get_mcp_path()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        with tempfile.TemporaryDirectory() as tmp:
            store_path = os.path.join(tmp, "mcp_store.json")
            broken_content = "not json"
            with open(store_path, "w", encoding="utf-8") as f:
                f.write(broken_content)

            # 1. Inspect without recovery
            req = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "aios.kernel_module.check",
                    "arguments": {"store_path": store_path}
                }
            }
            p.stdin.write(json.dumps(req) + "\n")
            p.stdin.flush()
            line = p.stdout.readline()
            resp = json.loads(line)
            res = json.loads(resp["result"]["content"][0]["text"])
            assert res["ok"] is False
            assert res["data"]["healthy"] is False

            # 2. Inspect with auto_recover
            req2 = {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "aios.kernel_module.check",
                    "arguments": {
                        "store_path": store_path,
                        "auto_recover": True
                    }
                }
            }
            p.stdin.write(json.dumps(req2) + "\n")
            p.stdin.flush()
            line2 = p.stdout.readline()
            resp2 = json.loads(line2)
            res2 = json.loads(resp2["result"]["content"][0]["text"])
            assert res2["ok"] is True
            assert res2["data"]["healthy"] is True
            assert res2["data"]["recovered"] is True

            print("PASS: test_mcp_check_tool")
    finally:
        p.stdin.close()
        p.terminate()
        p.wait()


def test_cross_surface_parity():
    with tempfile.TemporaryDirectory() as tmp:
        store_path = os.path.join(tmp, "parity_store.json")
        with open(store_path, "w", encoding="utf-8") as f:
            f.write("{ invalid json")

        # Recover using MCP
        p = subprocess.Popen(
            [get_mcp_path()],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            req = {
                "jsonrpc": "2.0",
                "id": 10,
                "method": "tools/call",
                "params": {
                    "name": "aios.kernel_module.check",
                    "arguments": {
                        "store_path": store_path,
                        "auto_recover": True
                    }
                }
            }
            p.stdin.write(json.dumps(req) + "\n")
            p.stdin.flush()
            line = p.stdout.readline()
            resp = json.loads(line)
            res = json.loads(resp["result"]["content"][0]["text"])
            assert res["ok"] is True
        finally:
            p.stdin.close()
            p.terminate()
            p.wait()

        # Check parity via CLI: CLI must immediately see the recovered store as healthy
        cli_res = run_cli("mod", "check", "--store", store_path, "--json")
        assert cli_res.returncode == 0
        cli_data = json.loads(cli_res.stdout)
        assert cli_data["data"]["healthy"] is True
    print("PASS: test_cross_surface_parity")


if __name__ == "__main__":
    test_cli_check_healthy()
    test_cli_check_corrupted_and_auto_recover()
    test_cli_check_human_output()
    test_mcp_check_tool()
    test_cross_surface_parity()
    print("ALL RECOVERY & VALIDATION INTEGRATION SMOKE TESTS PASSED.")
