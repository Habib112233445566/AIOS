#!/usr/bin/env python3
"""Integration Smoke Test for Kernel Module Management Observability (T-01676).

Validates CLI (`aiosh mod observability`) and MCP (`aios.kernel_module.observability`) surfaces
for report generation, live procfs aggregation, rule distributions, and policy metrics.
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


def test_cli_observability_default():
    res = run_cli("mod", "observability", "--json")
    assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
    data = json.loads(res.stdout)
    assert data["code"] == 0
    rep = data["data"]
    assert "total_loaded_modules" in rep
    assert "total_memory_bytes" in rep
    assert "state_breakdown" in rep
    assert "store_rules_count" in rep
    assert "rule_type_breakdown" in rep
    assert "autoload_modules_count" in rep
    assert "ref_count_distribution" in rep
    assert "policy_compliant_count" in rep
    assert "policy_violations_count" in rep
    print("PASS: test_cli_observability_default")


def test_cli_observability_with_mock_procfs():
    with tempfile.TemporaryDirectory() as td:
        store_path = os.path.join(td, "kernel_modules.json")
        proc_path = os.path.join(td, "modules")

        # Mock procfs
        with open(proc_path, "w") as f:
            f.write("overlay 151552 1 - Live 0x0\next4 983040 2 - Live 0x0\n")

        # Populate store
        res_bl = run_cli("mod", "blacklist", "floppy", "--store", store_path, "--json")
        assert res_bl.returncode == 0
        res_al = run_cli("mod", "autoload", "dummy_net", "--store", store_path, "--json")
        assert res_al.returncode == 0

        # Generate observability report
        res = run_cli(
            "mod", "observability",
            "--store", store_path,
            "--proc-modules", proc_path,
            "--json"
        )
        assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
        data = json.loads(res.stdout)
        rep = data["data"]
        assert rep["total_loaded_modules"] == 2
        assert rep["total_memory_bytes"] == 151552 + 983040
        assert rep["store_rules_count"] == 1
        assert rep["autoload_modules_count"] == 1
        assert rep["policy_compliant_count"] == 2
        assert rep["policy_violations_count"] == 0
        print("PASS: test_cli_observability_with_mock_procfs")


def test_mcp_observability_tool():
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
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "aios.kernel_module.observability",
                "arguments": {}
            }
        }
        p.stdin.write(json.dumps(req) + "\n")
        p.stdin.flush()
        line = p.stdout.readline()
        resp = json.loads(line)
        res = json.loads(resp["result"]["content"][0]["text"])
        assert res["ok"] is True
        assert res["tool"] == "aios.kernel_module.observability"
        rep = res["data"]
        assert "total_loaded_modules" in rep
        assert "ref_count_distribution" in rep
        print("PASS: test_mcp_observability_tool")
    finally:
        p.stdin.close()
        p.terminate()
        p.wait()


if __name__ == "__main__":
    test_cli_observability_default()
    test_cli_observability_with_mock_procfs()
    test_mcp_observability_tool()
    print("ALL OBSERVABILITY INTEGRATION TESTS PASSED.")
