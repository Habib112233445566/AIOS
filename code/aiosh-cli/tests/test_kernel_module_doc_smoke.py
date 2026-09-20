#!/usr/bin/env python3
"""Integration Smoke Test for Kernel Module Management Documentation (T-01686).

Validates CLI (`aiosh mod doc [list|get|search]`) and MCP (`aios.kernel_module.doc`)
surfaces for topic listing, markdown rendering, search scoring, and JSON schemas.
"""

import json
import os
import subprocess
import sys
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


def test_cli_doc_list():
    res = run_cli("mod", "doc", "list", "--json")
    assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
    data = json.loads(res.stdout)
    assert data["code"] == 0
    topics = data["data"]
    assert len(topics) >= 7
    ids = [t["id"] for t in topics]
    assert "modprobe-directives" in ids
    assert "cis-benchmark-hardening" in ids
    assert "lifecycle-workflows" in ids
    assert "observability-and-procfs" in ids
    assert "security-policy-and-pep" in ids
    assert "container-isolation" in ids
    assert "wireless-pentest" in ids
    print("PASS: test_cli_doc_list")


def test_cli_doc_get():
    res = run_cli("mod", "doc", "get", "cis-benchmark-hardening", "--json")
    assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
    data = json.loads(res.stdout)
    assert data["code"] == 0
    topic = data["data"]
    assert topic["id"] == "cis-benchmark-hardening"
    assert topic["category"] == "security"
    assert len(topic["sections"]) >= 3
    assert len(topic["references"]) >= 2
    print("PASS: test_cli_doc_get")


def test_cli_doc_search():
    res = run_cli("mod", "doc", "search", "cramfs", "--json")
    assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
    data = json.loads(res.stdout)
    assert data["code"] == 0
    results = data["data"]
    assert len(results) > 0
    assert results[0]["topic_id"] == "cis-benchmark-hardening"
    assert results[0]["score"] > 0
    print("PASS: test_cli_doc_search")


def test_cli_doc_markdown_rendering():
    res = run_cli("mod", "doc", "get", "container-isolation")
    assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
    assert "# Container & Sandbox Namespace Module Configuration" in res.stdout
    assert "Overlay Filesystem Optimization" in res.stdout
    assert "Authoritative References" in res.stdout
    print("PASS: test_cli_doc_markdown_rendering")


def test_mcp_doc_tool():
    p = subprocess.Popen(
        [get_mcp_path()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    try:
        # 1. Action: list
        req = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "aios.kernel_module.doc",
                "arguments": {"action": "list"}
            }
        }
        p.stdin.write(json.dumps(req) + "\n")
        p.stdin.flush()
        line = p.stdout.readline()
        resp = json.loads(line)
        res = json.loads(resp["result"]["content"][0]["text"])
        assert res["ok"] is True
        assert res["tool"] == "aios.kernel_module.doc"
        assert len(res["data"]) >= 7

        # 2. Action: get
        req2 = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "aios.kernel_module.doc",
                "arguments": {"action": "get", "topic": "wireless-pentest"}
            }
        }
        p.stdin.write(json.dumps(req2) + "\n")
        p.stdin.flush()
        line2 = p.stdout.readline()
        resp2 = json.loads(line2)
        res2 = json.loads(resp2["result"]["content"][0]["text"])
        assert res2["ok"] is True
        assert res2["data"]["id"] == "wireless-pentest"
        assert "markdown" in res2
        assert "ath9k_htc" in res2["markdown"]

        # 3. Action: search
        req3 = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "aios.kernel_module.doc",
                "arguments": {"action": "search", "query": "overlay"}
            }
        }
        p.stdin.write(json.dumps(req3) + "\n")
        p.stdin.flush()
        line3 = p.stdout.readline()
        resp3 = json.loads(line3)
        res3 = json.loads(resp3["result"]["content"][0]["text"])
        assert res3["ok"] is True
        assert len(res3["data"]) > 0
        assert any(r["topic_id"] == "container-isolation" for r in res3["data"])

        print("PASS: test_mcp_doc_tool")
    finally:
        p.stdin.close()
        p.terminate()
        p.wait()


if __name__ == "__main__":
    test_cli_doc_list()
    test_cli_doc_get()
    test_cli_doc_search()
    test_cli_doc_markdown_rendering()
    test_mcp_doc_tool()
    print("ALL DOCUMENTATION INTEGRATION TESTS PASSED.")
