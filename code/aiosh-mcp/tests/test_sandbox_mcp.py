#!/usr/bin/env python3
"""Comprehensive Unit and Smoke Tests for Sandbox MCP Tools (T-02434/T-02435).

Covers all 3 Sandbox Enforcement tools over MCP JSON-RPC 2.0 stdio:
1. `aios.sandbox.profiles`: Listing default profiles.
2. `aios.sandbox.probe`: Host kernel capability detection.
3. `aios.sandbox.exec`:
   - Valid command execution under permissive profile.
   - Validation failure: empty command.
   - Validation failure: directory traversal in cwd.
   - Validation failure: nonexistent profile.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _find_binary(names: list[str]) -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in names:
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return names[-1]


def get_mcp_binary() -> str:
    return _find_binary(["aiosh-mcp.exe", "aiosh-mcp"])


def run_mcp(payload: dict, timeout_s: int = 30) -> dict:
    p = subprocess.Popen(
        [get_mcp_binary()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
    )
    try:
        stdout, _ = p.communicate(json.dumps(payload) + "\n", timeout=timeout_s)
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
        raise TimeoutError(f"aiosh-mcp timed out after {timeout_s}s")
    finally:
        if p.poll() is None:
            p.kill()
            p.wait()

    assert p.returncode == 0, f"aiosh-mcp exited with returncode {p.returncode}"
    return json.loads(stdout.strip())


def call_mcp_tool(tool_name: str, arguments: dict | None = None, timeout_s: int = 30) -> dict:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments or {}},
    }
    resp = run_mcp(payload, timeout_s=timeout_s)
    if "error" in resp:
        return {"ok": False, "error": resp["error"]}
    res = resp.get("result", {})
    if "structuredContent" in res and "result" in res["structuredContent"]:
        return res["structuredContent"]["result"]
    elif "content" in res and res["content"]:
        try:
            return json.loads(res["content"][0]["text"])
        except Exception:
            return {"ok": False, "text": res["content"][0]["text"]}
    return res


def list_mcp_tools(timeout_s: int = 30) -> list[dict]:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {},
    }
    resp = run_mcp(payload, timeout_s=timeout_s)
    return resp.get("result", {}).get("tools", [])


def test_mcp_sandbox_tool_registration():
    tools = list_mcp_tools()
    tool_names = {t["name"] for t in tools}
    assert "aios.sandbox.profiles" in tool_names
    assert "aios.sandbox.probe" in tool_names
    assert "aios.sandbox.exec" in tool_names


def test_mcp_sandbox_profiles():
    res = call_mcp_tool("aios.sandbox.profiles")
    assert res.get("ok") is True
    profiles = res.get("profiles", [])
    names = {p["name"] for p in profiles}
    assert "standard" in names
    assert "strict" in names
    assert "permissive" in names


def test_mcp_sandbox_probe():
    res = call_mcp_tool("aios.sandbox.probe")
    assert res.get("ok") is True
    caps = res.get("capabilities", {})
    assert "platform" in caps
    assert "landlock_supported" in caps
    assert "seccomp_bpf_supported" in caps
    assert "no_new_privs_supported" in caps


def test_mcp_sandbox_exec_success():
    res = call_mcp_tool("aios.sandbox.exec", {
        "command": sys.executable,
        "args": ["-c", "print('mcp_sandbox_ok')"],
        "profile": "permissive"
    })
    assert res.get("ok") is True
    exec_res = res.get("result", {})
    assert exec_res.get("exit_code") == 0
    assert "mcp_sandbox_ok" in exec_res.get("stdout", "")


def test_mcp_sandbox_exec_empty_command():
    res = call_mcp_tool("aios.sandbox.exec", {
        "command": "   ",
    })
    assert res.get("ok") is False
    assert "ERR_SANDBOX_EMPTY_COMMAND" in str(res.get("error", ""))


def test_mcp_sandbox_exec_traversal_cwd():
    res = call_mcp_tool("aios.sandbox.exec", {
        "command": sys.executable,
        "cwd": "../outside"
    })
    assert res.get("ok") is False
    assert "ERR_SANDBOX_INVALID_PATH" in str(res.get("error", ""))


def test_mcp_sandbox_exec_unknown_profile():
    res = call_mcp_tool("aios.sandbox.exec", {
        "command": sys.executable,
        "profile": "unknown_profile_xyz"
    })
    assert res.get("ok") is False
    assert "ERR_SANDBOX_PROFILE_NOT_FOUND" in str(res.get("error", ""))


def test_mcp_sandbox_audit_persistence():
    import sqlite3
    import os
    res = call_mcp_tool("aios.sandbox.exec", {
        "command": sys.executable,
        "args": ["-c", "print('audit_persisted_mcp')"],
        "profile": "permissive"
    })
    assert res.get("ok") is True

    ai_home = os.environ.get("AI_HOME", os.path.expanduser("~/.aiosh"))
    db_path = os.path.join(ai_home, "aiosh.db")
    if os.path.exists(db_path):
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()
        cur.execute("SELECT id, tool, command, outcome FROM audit_ring WHERE tool='aios.sandbox.exec' ORDER BY id DESC LIMIT 1")
        row = cur.fetchone()
        conn.close()
        assert row is not None
        assert row[1] == "aios.sandbox.exec"


def test_mcp_sandbox_config():
    res = call_mcp_tool("aios.sandbox.config")
    assert res.get("ok") is True
    cfg = res.get("config", {})
    assert cfg.get("version") == "1.0.0"
    assert cfg.get("default_profile_name") == "standard"
    assert cfg.get("max_output_capture_bytes") == 10 * 1024 * 1024


if __name__ == "__main__":
    test_mcp_sandbox_tool_registration()
    test_mcp_sandbox_profiles()
    test_mcp_sandbox_probe()
    test_mcp_sandbox_exec_success()
    test_mcp_sandbox_exec_empty_command()
    test_mcp_sandbox_exec_traversal_cwd()
    test_mcp_sandbox_exec_unknown_profile()
    test_mcp_sandbox_audit_persistence()
    test_mcp_sandbox_config()
    print("ALL 9 SANDBOX MCP TESTS PASSED!")

