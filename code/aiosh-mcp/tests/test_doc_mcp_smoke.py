#!/usr/bin/env python3
"""MCP Smoke & Unit Test for Documentation Index Control (T-00445)"""

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
    assert "aios.doc.index.get" in tool_names, "Missing aios.doc.index.get"
    assert "aios.doc.check" in tool_names, "Missing aios.doc.check"
    assert "aios.doc.search" in tool_names, "Missing aios.doc.search"
    print("PASS: aios.doc tools present in tools/list")

def test_mcp_doc_index_get():
    payload = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "aios.doc.index.get",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {})
    assert not result.get("isError"), f"MCP Error: {result}"
    structured = result.get("structuredContent", {}).get("result", {})
    assert structured.get("ok") is True
    manifest = structured.get("manifest", {})
    assert "entries" in manifest
    assert len(manifest["entries"]) > 0
    print("PASS: aios.doc.index.get")

def test_mcp_doc_check():
    payload = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "aios.doc.check",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {})
    assert not result.get("isError"), f"MCP Error: {result}"
    structured = result.get("structuredContent", {}).get("result", {})
    assert structured.get("ok") is True
    report = structured.get("report", {})
    assert report.get("is_valid") is True
    assert report.get("total_links_checked", 0) > 0
    print("PASS: aios.doc.check")

def test_mcp_doc_search():
    payload = {
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "aios.doc.search",
            "arguments": {
                "query": "task"
            }
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {})
    assert not result.get("isError"), f"MCP Error: {result}"
    structured = result.get("structuredContent", {}).get("result", {})
    assert structured.get("ok") is True
    matches = structured.get("matches", [])
    assert len(matches) > 0
    print("PASS: aios.doc.search")

def test_mcp_doc_search_missing_query_negative():
    payload = {
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "aios.doc.search",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    result = res.get("result", {})
    assert result.get("isError") is True or result.get("structuredContent", {}).get("result", {}).get("ok") is False
    print("PASS: aios.doc.search missing query negative test")

if __name__ == "__main__":
    test_mcp_tools_list()
    test_mcp_doc_index_get()
    test_mcp_doc_check()
    test_mcp_doc_search()
    test_mcp_doc_search_missing_query_negative()
    print("PASS: test_doc_mcp_smoke.py")
