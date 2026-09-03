#!/usr/bin/env python3
"""MCP Smoke & Integration Test for Distro Selection & Justification (T-01033..T-01036)."""

import json
import subprocess
import sys
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

def test_mcp_distro_tools_manifest():
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    }
    res = run_mcp(payload)
    tools = res.get("result", {}).get("tools", [])
    names = {t["name"] for t in tools}
    required = {
        "aios.distro.list",
        "aios.distro.show",
        "aios.distro.evaluate",
        "aios.distro.recommend",
    }
    missing = required - names
    assert not missing, f"Missing MCP tools: {missing}"
    print("PASS: aiosh-mcp tools/list includes all 4 distro tools")

def get_tool_result(res):
    if "structuredContent" in res.get("result", {}):
        return res["result"]["structuredContent"].get("result", {})
    if "content" in res.get("result", {}) and res["result"]["content"]:
        return json.loads(res["result"]["content"][0]["text"])
    return res.get("result", {})

def test_mcp_distro_list_call():
    payload = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "aios.distro.list",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    data = get_tool_result(res)
    assert "count" in data, f"Missing count in {data}"
    assert data["count"] >= 2
    assert "profiles" in data
    print("PASS: aiosh-mcp tools/call aios.distro.list")

def test_mcp_distro_show_call():
    payload = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "aios.distro.show",
            "arguments": {
                "id": "debian-12-minimal-x86_64"
            }
        }
    }
    res = run_mcp(payload)
    data = get_tool_result(res)
    assert data.get("ok") is True, f"Unexpected data: {data}"
    profile = data.get("profile", {})
    assert profile.get("id") == "debian-12-minimal-x86_64"
    assert profile.get("family") == "Debian"
    print("PASS: aiosh-mcp tools/call aios.distro.show")

def test_mcp_distro_show_missing_id():
    payload = {
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "aios.distro.show",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    is_err = "error" in res or res.get("result", {}).get("isError") is True
    assert is_err, f"Expected error envelope, got: {res}"
    data = get_tool_result(res)
    assert data.get("ok") is False or "error" in res
    print("PASS: aiosh-mcp tools/call aios.distro.show missing id rejected with error envelope")

def test_mcp_distro_evaluate_call():
    payload = {
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "aios.distro.evaluate",
            "arguments": {
                "id": "alpine-319-container-x86_64"
            }
        }
    }
    res = run_mcp(payload)
    data = get_tool_result(res)
    assert data.get("ok") is True
    eval_item = data.get("evaluation", {})
    assert eval_item.get("profile_id") == "alpine-319-container-x86_64"
    assert "overall_score" in eval_item
    print("PASS: aiosh-mcp tools/call aios.distro.evaluate")

def test_mcp_distro_recommend_call():
    payload = {
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "aios.distro.recommend",
            "arguments": {}
        }
    }
    res = run_mcp(payload)
    data = get_tool_result(res)
    assert data.get("ok") is True
    profile = data.get("profile", {})
    assert profile.get("id") == "debian-12-minimal-x86_64"
    assert profile.get("recommended") is True
    print("PASS: aiosh-mcp tools/call aios.distro.recommend")

def test_mcp_distro_show_not_found():
    payload = {
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "aios.distro.show",
            "arguments": {
                "id": "nonexistent-profile-xyz"
            }
        }
    }
    res = run_mcp(payload)
    data = get_tool_result(res)
    assert data.get("ok") is False or "error" in data
    print("PASS: aiosh-mcp tools/call aios.distro.show nonexistent profile returns ok: false")

def main():
    test_mcp_distro_tools_manifest()
    test_mcp_distro_list_call()
    test_mcp_distro_show_call()
    test_mcp_distro_show_missing_id()
    test_mcp_distro_show_not_found()
    test_mcp_distro_evaluate_call()
    test_mcp_distro_recommend_call()
    print("\nALL DISTRO MCP SMOKE TESTS PASSED!")
    return 0

if __name__ == "__main__":
    sys.exit(main())
