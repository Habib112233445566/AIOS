import json
import subprocess
import pytest
import sys
from pathlib import Path

MCP_BIN = Path("code/aiosh-rust/target/debug/aiosh-mcp.exe")

def test_mcp_binary_exists():
    assert MCP_BIN.exists(), f"Binary {MCP_BIN} must exist"

def run_mcp_call(method: str, params: dict):
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": method,
            "arguments": params
        }
    }
    p = subprocess.Popen(
        [str(MCP_BIN)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    try:
        stdout, stderr = p.communicate(input=json.dumps(req) + "\n", timeout=10)
    finally:
        if p.poll() is None:
            p.kill()
            p.wait()

    assert p.returncode == 0
    res_line = [l for l in stdout.splitlines() if l.strip()][0]
    raw = json.loads(res_line)
    res = raw.get("result", {})
    if "content" in res and res["content"]:
        return json.loads(res["content"][0]["text"])
    return raw

def test_mcp_sandbox_exec_success():
    res = run_mcp_call("aios.sandbox.exec", {
        "command": sys.executable,
        "args": ["-c", "print('automated_unit_test_ok')"],
        "profile": "permissive"
    })
    assert res.get("ok") is True
    exec_res = res.get("result", {})
    assert exec_res.get("exit_code") == 0
    assert "automated_unit_test_ok" in exec_res.get("stdout", "")

def test_mcp_sandbox_exec_failure_code_propagation():
    res = run_mcp_call("aios.sandbox.exec", {
        "command": sys.executable,
        "args": ["-c", "import sys; sys.exit(77)"],
        "profile": "permissive"
    })
    assert res.get("ok") is False
    exec_res = res.get("result", {})
    assert exec_res.get("exit_code") == 77

def test_mcp_sandbox_exec_nonexistent_binary():
    res = run_mcp_call("aios.sandbox.exec", {
        "command": "nonexistent_executable_123456789",
        "args": [],
        "profile": "permissive"
    })
    assert res.get("ok") is False
    exec_res = res.get("result", {})
    assert exec_res.get("exit_code") == 127

def test_mcp_sandbox_exec_traversal_cwd_rejected():
    res = run_mcp_call("aios.sandbox.exec", {
        "command": sys.executable,
        "args": ["-c", "print('should_fail')"],
        "cwd": "../../etc",
        "profile": "permissive"
    })
    assert res.get("ok") is False
    assert "ERR_SANDBOX_INVALID_PATH" in str(res.get("error", ""))
