#!/usr/bin/env python3
"""MCP Smoke & Unit Test for Repository Health (T-00645)."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

def get_mcp_binary():
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

def run_mcp(payload, timeout_s=30):
    bin_path = get_mcp_binary()
    p = subprocess.Popen(
        [bin_path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True
    )
    try:
        stdout, _ = p.communicate(json.dumps(payload) + "\n", timeout=timeout_s)
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
        print(f"FAIL: aiosh-mcp timed out after {timeout_s}s")
        sys.exit(1)
    if p.returncode != 0:
        print(f"FAIL: aiosh-mcp returned {p.returncode}")
        sys.exit(1)
    try:
        return json.loads(stdout.strip())
    except Exception as e:
        print(f"FAIL: invalid JSON from aiosh-mcp: {e}")
        print(stdout)
        sys.exit(1)

def test_mcp_tools_list_repo_health():
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    }
    res = run_mcp(payload)
    tools = res.get("result", {}).get("tools", [])
    tool_names = [t.get("name") for t in tools]
    assert "aios.repo.health" in tool_names, "Missing aios.repo.health in tools/list"
    
    repo_tool = next(t for t in tools if t.get("name") == "aios.repo.health")
    assert "repo_path" in repo_tool.get("inputSchema", {}).get("properties", {})
    print("PASS: aios.repo.health present and valid in tools/list")

def test_mcp_repo_health_default_path():
    payload = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "aios.repo.health",
            "arguments": {
                "repo_path": str(ROOT)
            }
        }
    }
    res = run_mcp(payload)
    assert not res.get("result", {}).get("isError", False)
    content = json.loads(res["result"]["content"][0]["text"])
    assert content.get("ok") is True
    assert content.get("tool") == "aios.repo.health"
    report = content.get("report")
    assert report is not None
    assert report.get("total_checks") >= 3
    print("PASS: aios.repo.health tool call on repository root")

def test_mcp_repo_health_custom_temp_repo():
    with tempfile.TemporaryDirectory() as td:
        sec_path = Path(td) / "SECURITY.md"
        sec_path.write_text("# Security Policy\n\nValid security policy for MCP test.\nNo disclosure issues found.\n")
        
        payload = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "aios.repo.health",
                "arguments": {
                    "repo_path": td
                }
            }
        }
        res = run_mcp(payload)
        assert not res.get("result", {}).get("isError", False)
        content = json.loads(res["result"]["content"][0]["text"])
        assert content.get("ok") is True
        assert content.get("report", {}).get("total_checks") == 3
        print("PASS: aios.repo.health tool call on temp directory")

def main():
    test_mcp_tools_list_repo_health()
    test_mcp_repo_health_default_path()
    test_mcp_repo_health_custom_temp_repo()
    print("\nALL MCP REPO HEALTH SMOKE TESTS PASSED!")

if __name__ == "__main__":
    main()
