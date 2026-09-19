#!/usr/bin/env python3
"""Integration Smoke Test for Kernel Module Management Security Policy (T-01666).

Validates CLI (`aiosh mod policy`) and MCP (`aios.kernel_module.policy`) surfaces
for policy inspection, module evaluation, store evaluation, and violation detection.
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


def test_cli_policy_inspect():
    res = run_cli("mod", "policy", "--json")
    assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
    data = json.loads(res.stdout)
    assert data["code"] == 0
    assert data["data"]["mode"] == "enforcing"
    assert "cramfs" in data["data"]["prohibited_modules"]
    assert "ext4" in data["data"]["protected_modules"]
    print("PASS: test_cli_policy_inspect")


def test_cli_policy_evaluate_module():
    # 1. Allowed module
    res_ok = run_cli("mod", "policy", "dummy_net", "--json")
    assert res_ok.returncode == 0, f"Expected 0, got {res_ok.returncode}: {res_ok.stderr}"
    data_ok = json.loads(res_ok.stdout)
    assert data_ok["code"] == 0
    assert data_ok["data"]["allowed"] is True

    # 2. Prohibited module
    res_bad = run_cli("mod", "policy", "cramfs", "--json")
    assert res_bad.returncode == 1, f"Expected 1, got {res_bad.returncode}"
    data_bad = json.loads(res_bad.stdout)
    assert data_bad["code"] == 1
    assert data_bad["data"]["allowed"] is False
    assert any(v["rule_id"] == "SP-KM2-PROHIBITED-AUTOLOAD" for v in data_bad["data"]["violations"])
    print("PASS: test_cli_policy_evaluate_module")


def test_cli_policy_evaluate_store():
    with tempfile.TemporaryDirectory() as td:
        store_path = os.path.join(td, "kernel_modules.json")

        # Create store with a clean module
        res1 = run_cli("mod", "blacklist", "floppy", "--store", store_path, "--json")
        assert res1.returncode == 0

        # Evaluate clean store
        res_eval = run_cli("mod", "policy", "--evaluate-store", "--store", store_path, "--json")
        assert res_eval.returncode == 0
        data_eval = json.loads(res_eval.stdout)
        assert data_eval["code"] == 0
        assert data_eval["data"]["allowed"] is True

        # Now add options for a prohibited module into store
        res_opt = run_cli("mod", "options", "cramfs", "debug=1", "--store", store_path, "--json")
        assert res_opt.returncode == 0

        # Evaluate store with prohibited module option
        res_eval_bad = run_cli("mod", "policy", "--evaluate-store", "--store", store_path, "--json")
        assert res_eval_bad.returncode == 1
        data_eval_bad = json.loads(res_eval_bad.stdout)
        assert data_eval_bad["code"] == 1
        assert data_eval_bad["data"]["allowed"] is False
        print("PASS: test_cli_policy_evaluate_store")


def test_mcp_policy_tool():
    p = subprocess.Popen(
        [get_mcp_path()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        # 1. Inspect policy
        req_inspect = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "aios.kernel_module.policy",
                "arguments": {}
            }
        }
        p.stdin.write(json.dumps(req_inspect) + "\n")
        p.stdin.flush()
        line = p.stdout.readline()
        resp = json.loads(line)
        res = json.loads(resp["result"]["content"][0]["text"])
        assert res["ok"] is True
        assert res["data"]["mode"] == "enforcing"

        # 2. Evaluate prohibited module
        req_eval = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "aios.kernel_module.policy",
                "arguments": {
                    "module": "cramfs"
                }
            }
        }
        p.stdin.write(json.dumps(req_eval) + "\n")
        p.stdin.flush()
        line2 = p.stdout.readline()
        resp2 = json.loads(line2)
        res2 = json.loads(resp2["result"]["content"][0]["text"])
        assert res2["ok"] is False
        assert res2["data"]["allowed"] is False
        assert any(v["rule_id"] == "SP-KM2-PROHIBITED-AUTOLOAD" for v in res2["data"]["violations"])

        print("PASS: test_mcp_policy_tool")
    finally:
        p.stdin.close()
        p.terminate()
        p.wait()


if __name__ == "__main__":
    test_cli_policy_inspect()
    test_cli_policy_evaluate_module()
    test_cli_policy_evaluate_store()
    test_mcp_policy_tool()
    print("ALL POLICY INTEGRATION TESTS PASSED.")
