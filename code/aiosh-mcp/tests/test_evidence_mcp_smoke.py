#!/usr/bin/env python3
"""MCP Smoke & Unit Test for Evidence & Audit Trail (T-00545)."""

import json
import os
import subprocess
import sys
from pathlib import Path

def get_mcp_binary():
    candidates = [
        Path("code/aiosh-rust/target/debug/aiosh-mcp.exe"),
        Path("code/aiosh-rust/target/debug/aiosh-mcp"),
        Path("target/debug/aiosh-mcp.exe"),
        Path("target/debug/aiosh-mcp"),
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

def test_mcp_tools_list():
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    }
    res = run_mcp(payload)
    tools = res.get("result", {}).get("tools", [])
    tool_names = [t.get("name") for t in tools]
    assert "aios.evidence.verify" in tool_names, "Missing aios.evidence.verify"
    assert "aios.evidence.hash" in tool_names, "Missing aios.evidence.hash"
    assert "aios.evidence.scan" in tool_names, "Missing aios.evidence.scan"
    print("PASS: aios.evidence tools present in tools/list")

def test_mcp_evidence_hash():
    payload = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "aios.evidence.hash",
            "arguments": {
                "file_path": "docs/README.md"
            }
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {}).get("structuredContent", {}).get("result", {})
    assert result.get("ok") is True
    assert result.get("tool") == "aios.evidence.hash"
    assert "sha256" in result
    assert len(result["sha256"]) == 64
    print("PASS: aios.evidence.hash execution")

def test_mcp_evidence_hash_missing_file_error():
    payload = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "aios.evidence.hash",
            "arguments": {
                "file_path": "non_existent_file_xyz.md"
            }
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {}).get("structuredContent", {}).get("result", {})
    assert result.get("ok") is False
    assert "error" in result
    print("PASS: aios.evidence.hash missing file error")

def test_mcp_evidence_hash_missing_arg_error():
    payload = {
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "aios.evidence.hash",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {}).get("structuredContent", {}).get("result", {})
    assert result.get("ok") is False
    assert "error" in result
    print("PASS: aios.evidence.hash missing arg error")

def test_mcp_evidence_verify():
    payload = {
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "aios.evidence.verify",
            "arguments": {
                "repo_path": "."
            }
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {}).get("structuredContent", {}).get("result", {})
    assert result.get("ok") is True
    assert result.get("tool") == "aios.evidence.verify"
    print("PASS: aios.evidence.verify execution")

def test_mcp_evidence_scan():
    payload = {
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "aios.evidence.scan",
            "arguments": {
                "repo_path": "."
            }
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {}).get("structuredContent", {}).get("result", {})
    assert result.get("ok") is True
    assert result.get("tool") == "aios.evidence.scan"
    assert "records" in result
    assert len(result["records"]) > 0
    print("PASS: aios.evidence.scan execution")

def test_mcp_evidence_scan_filtered():
    payload = {
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "aios.evidence.scan",
            "arguments": {
                "repo_path": ".",
                "task_id": 501
            }
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {}).get("structuredContent", {}).get("result", {})
    assert result.get("ok") is True
    assert result.get("tool") == "aios.evidence.scan"
    assert len(result.get("records", [])) >= 1
    for r in result["records"]:
        assert r["task_id"] == 501
    print("PASS: aios.evidence.scan filtered by task")

def test_mcp_evidence_scan_missing_dir_error():
    payload = {
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {
            "name": "aios.evidence.scan",
            "arguments": {
                "repo_path": "non_existent_repo_dir_xyz"
            }
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {}).get("structuredContent", {}).get("result", {})
    assert result.get("ok") is False
    assert "error" in result
    print("PASS: aios.evidence.scan missing dir error")

def main():
    test_mcp_tools_list()
    test_mcp_evidence_hash()
    test_mcp_evidence_hash_missing_file_error()
    test_mcp_evidence_hash_missing_arg_error()
    test_mcp_evidence_verify()
    test_mcp_evidence_scan()
    test_mcp_evidence_scan_filtered()
    test_mcp_evidence_scan_missing_dir_error()
    print("All 8 evidence MCP unit and smoke tests passed successfully!")

if __name__ == "__main__":
    main()
