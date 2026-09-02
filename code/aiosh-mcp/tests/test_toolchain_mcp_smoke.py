#!/usr/bin/env python3
"""MCP Smoke & Unit Test for Toolchain Pinning (T-00365)"""

import json
import subprocess
import sys
import os

def run_mcp(payload, timeout_s=30):
    p = subprocess.Popen(
        ["code/aiosh-rust/target/debug/aiosh-mcp.exe"],
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

def test_mcp_config():
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "aios.toolchain.config.get",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {})
    assert not result.get("isError"), f"MCP Error: {result}"
    structured = result.get("structuredContent", {}).get("result", {})
    assert structured.get("ok") is True
    assert structured.get("tool") == "aios.toolchain.config.get"
    assert "rust_version" in structured.get("config", {})
    assert "python_version" in structured.get("config", {})
    print("PASS: aios.toolchain.config.get")

def test_mcp_check():
    payload = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "aios.toolchain.check",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {})
    assert not result.get("isError"), f"MCP Error: {result}"
    structured = result.get("structuredContent", {}).get("result", {})
    assert structured.get("ok") is True
    assert structured.get("tool") == "aios.toolchain.check"
    print("PASS: aios.toolchain.check")

def test_mcp_unknown_tool_fails():
    payload = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "aios.toolchain.nonexistent",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {})
    assert result.get("isError") is True or "error" in res, f"Expected error for unknown tool: {res}"
    print("PASS: aios.toolchain unknown tool negative test")

if __name__ == "__main__":
    if not os.path.exists("code/aiosh-rust/target/debug/aiosh-mcp.exe"):
        subprocess.run(["cargo", "build", "--bin", "aiosh-mcp"], cwd="code/aiosh-rust", check=True)
    test_mcp_config()
    test_mcp_check()
    test_mcp_unknown_tool_fails()
    print("PASS: test_toolchain_mcp_smoke.py")
