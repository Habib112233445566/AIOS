#!/usr/bin/env python3
"""MCP Smoke & Unit Test for User Session Bootstrap Surface (T-01435).

Tests JSON-RPC 2.0 protocol over stdio for:
- aios.session.validate
- aios.session.list
- aios.session.get
- aios.session.action
- aios.session.create
Asserts valid inputs, invalid inputs, boundary values, state persistence, and failure modes.
"""

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
        text=True,
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


def call_mcp_tool(tool_name, arguments):
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments,
        },
    }
    resp = run_mcp(payload)
    if "error" in resp:
        return {"ok": False, "error": resp["error"]}
    res = resp.get("result", {})
    if "structuredContent" in res and "result" in res["structuredContent"]:
        data = res["structuredContent"]["result"]
    elif "content" in res and res["content"]:
        try:
            data = json.loads(res["content"][0]["text"])
        except Exception:
            data = {"ok": False, "text": res["content"][0]["text"]}
    else:
        data = dict(res)
    if res.get("isError") is True and "ok" not in data:
        data["ok"] = False
    return data


def test_manifest():
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {},
    }
    resp = run_mcp(payload)
    tools = resp.get("result", {}).get("tools", [])
    names = {t["name"] for t in tools}
    required = {
        "aios.session.validate",
        "aios.session.list",
        "aios.session.get",
        "aios.session.action",
        "aios.session.create",
    }
    missing = required - names
    if missing:
        print(f"FAIL: tools/list missing session tools: {missing}")
        sys.exit(1)
    print("PASS: tools/list contains all 5 aios.session.* tools")


def test_session_validate():
    # 1. Valid ID
    res = call_mcp_tool("aios.session.validate", {"session_id": "sess-01"})
    assert res.get("ok") is True and res.get("valid") is True, f"Failed valid ID: {res}"

    # 2. Invalid ID (path traversal)
    res = call_mcp_tool("aios.session.validate", {"session_id": "../bad"})
    assert res.get("ok") is False, f"Expected invalid ID failure: {res}"

    # 3. Valid username
    res = call_mcp_tool("aios.session.validate", {"username": "kali"})
    assert res.get("ok") is True and res.get("valid") is True, f"Failed valid user: {res}"

    # 4. Invalid username
    res = call_mcp_tool("aios.session.validate", {"username": "Kali_Invalid!"})
    assert res.get("ok") is False, f"Expected invalid username failure: {res}"

    # 5. Valid spec
    valid_spec = {
        "session_id": "sess-x11-01",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "x11",
        "session_class": "user",
        "seat": "seat0",
        "vtnr": 7,
        "display": ":0",
        "remote_host": None,
        "environment": {"XDG_RUNTIME_DIR": "/run/user/1000"},
    }
    res = call_mcp_tool("aios.session.validate", {"spec": valid_spec})
    assert res.get("ok") is True and res.get("valid") is True, f"Failed valid spec: {res}"

    # 6. Invalid spec (missing display for X11)
    invalid_spec = dict(valid_spec)
    invalid_spec["display"] = None
    res = call_mcp_tool("aios.session.validate", {"spec": invalid_spec})
    assert res.get("ok") is False, f"Expected invalid spec failure: {res}"

    # 7. Missing arguments
    res = call_mcp_tool("aios.session.validate", {})
    assert res.get("ok") is False, f"Expected missing arguments failure: {res}"

    print("PASS: aios.session.validate (valid, invalid, boundary, missing)")


def test_session_list_and_get():
    # 1. List default sessions
    res = call_mcp_tool("aios.session.list", {})
    assert res.get("ok") is True, f"List failed: {res}"
    assert res.get("count", 0) >= 1, f"Expected at least 1 session: {res}"
    sessions = res.get("sessions", [])
    assert any(s["session_id"] == "greeter-seat0" for s in sessions), "greeter-seat0 not found in list"

    # 2. Filter by seat
    res = call_mcp_tool("aios.session.list", {"seat": "seat0"})
    assert res.get("ok") is True, f"List by seat failed: {res}"

    # 3. Filter by non-existent user
    res = call_mcp_tool("aios.session.list", {"username": "nonexistentuser"})
    assert res.get("ok") is True and res.get("count") == 0, f"Expected 0 count: {res}"

    # 4. Get greeter-seat0
    res = call_mcp_tool("aios.session.get", {"session_id": "greeter-seat0"})
    assert res.get("ok") is True, f"Get failed: {res}"
    status = res.get("status", {})
    assert status.get("username") == "lightdm", f"Expected lightdm user: {status}"

    # 5. Get non-existent session
    res = call_mcp_tool("aios.session.get", {"session_id": "ghost-session"})
    assert res.get("ok") is False, f"Expected failure for ghost session: {res}"

    # 6. Missing session_id
    res = call_mcp_tool("aios.session.get", {})
    assert res.get("ok") is False, f"Expected failure for missing session_id: {res}"

    print("PASS: aios.session.list & aios.session.get (filtering, inspection, error modes)")


def test_session_action():
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
        store_path = tf.name

    try:
        # 1. Action lock with persistent store
        res = call_mcp_tool("aios.session.action", {"session_id": "greeter-seat0", "action": "lock", "store_path": store_path})
        assert res.get("ok") is True, f"Action lock failed: {res}"
        report = res.get("report", {})
        assert report.get("new_state") == "locked", f"Expected locked new_state: {report}"

        # 2. Action unlock with persistent store
        res = call_mcp_tool("aios.session.action", {"session_id": "greeter-seat0", "action": "unlock", "store_path": store_path})
        assert res.get("ok") is True, f"Action unlock failed: {res}"
        report = res.get("report", {})
        assert report.get("new_state") == "active", f"Expected active new_state: {report}"

        # 3. Unknown action
        res = call_mcp_tool("aios.session.action", {"session_id": "greeter-seat0", "action": "self_destruct"})
        assert res.get("ok") is False, f"Expected unknown action failure: {res}"

        # 4. Non-existent session
        res = call_mcp_tool("aios.session.action", {"session_id": "ghost", "action": "lock"})
        assert res.get("ok") is False, f"Expected failure on ghost session: {res}"

        # 5. Missing action
        res = call_mcp_tool("aios.session.action", {"session_id": "greeter-seat0"})
        assert res.get("ok") is False, f"Expected failure on missing action: {res}"

        print("PASS: aios.session.action (lock, unlock, unknown action, missing params)")
    finally:
        if os.path.exists(store_path):
            os.unlink(store_path)


def test_session_create_and_persistence():
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
        store_path = tf.name

    try:
        agent_spec = {
            "session_id": "agent-copilot-smoke",
            "username": "kali",
            "uid": 1000,
            "gid": 1000,
            "session_type": "ai_agent",
            "session_class": "agent",
            "seat": "seat0",
            "vtnr": 1,
            "display": None,
            "remote_host": None,
            "environment": {"AIOS_AGENT": "1"},
        }

        # 1. Create session into custom store
        res = call_mcp_tool("aios.session.create", {"spec": agent_spec, "store_path": store_path})
        assert res.get("ok") is True, f"Create session failed: {res}"
        assert res.get("session_id") == "agent-copilot-smoke", f"Mismatched session_id: {res}"
        assert res.get("status", {}).get("state") == "initializing", f"Expected initializing state: {res}"

        # 2. Get session from persistent store
        res = call_mcp_tool("aios.session.get", {"session_id": "agent-copilot-smoke", "store_path": store_path})
        assert res.get("ok") is True, f"Get created session failed: {res}"
        assert res.get("status", {}).get("session_id") == "agent-copilot-smoke", f"Status mismatch: {res}"

        # 3. Create duplicate session fails
        res = call_mcp_tool("aios.session.create", {"spec": agent_spec, "store_path": store_path})
        assert res.get("ok") is False, f"Expected duplicate create failure: {res}"
        assert "already exists" in str(res.get("error", "")), f"Expected 'already exists' in error: {res}"

        # 4. Create with invalid spec fails
        bad_spec = dict(agent_spec)
        bad_spec["session_id"] = "bad/id/with/slashes"
        res = call_mcp_tool("aios.session.create", {"spec": bad_spec, "store_path": store_path})
        assert res.get("ok") is False, f"Expected invalid spec failure: {res}"

        # 5. Missing spec parameter fails
        res = call_mcp_tool("aios.session.create", {"store_path": store_path})
        assert res.get("ok") is False, f"Expected missing spec failure: {res}"

        print("PASS: aios.session.create & persistence (create, get, duplicate rejection, invalid spec)")
    finally:
        if os.path.exists(store_path):
            os.unlink(store_path)


def test_cross_surface_cli_mcp_parity():
    cli_bin = ROOT / "code/aiosh-rust/target/debug/aiosh.exe"
    if not cli_bin.exists():
        cli_bin = ROOT / "code/aiosh-rust/target/debug/aiosh"
    if not cli_bin.exists():
        print("SKIP: aiosh binary not found for cross-surface test")
        return

    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
        store_path = tf.name

    try:
        # 1. Create via CLI
        cli_spec = {
            "session_id": "cross-surface-sess",
            "username": "kali",
            "uid": 1000,
            "gid": 1000,
            "session_type": "wayland",
            "session_class": "user",
            "seat": "seat0",
            "vtnr": 2,
            "display": ":1",
            "remote_host": None,
            "environment": {"XDG_RUNTIME_DIR": "/run/user/1000"},
        }
        cp = subprocess.run(
            [str(cli_bin), "session", "create", json.dumps(cli_spec), "--store", store_path, "--json"],
            capture_output=True,
            text=True,
        )
        assert cp.returncode == 0, f"CLI create failed: {cp.stderr}"

        # 2. Query via MCP
        res = call_mcp_tool("aios.session.get", {"session_id": "cross-surface-sess", "store_path": store_path})
        assert res.get("ok") is True, f"MCP query failed: {res}"
        assert res.get("status", {}).get("username") == "kali", f"Mismatched user: {res}"
        assert res.get("status", {}).get("state") == "initializing", f"Expected initializing state: {res}"

        # 3. Authenticate via MCP (Initializing -> Authenticating)
        res = call_mcp_tool("aios.session.action", {"session_id": "cross-surface-sess", "action": "authenticate", "store_path": store_path})
        assert res.get("ok") is True, f"MCP authenticate failed: {res}"
        assert res.get("report", {}).get("new_state") == "authenticating", f"Expected authenticating state: {res}"

        # 4. Activate via CLI (Authenticating -> Active)
        cp = subprocess.run(
            [str(cli_bin), "session", "activate", "cross-surface-sess", "--store", store_path, "--json"],
            capture_output=True,
            text=True,
        )
        assert cp.returncode == 0, f"CLI activate failed: {cp.stderr}"

        # 5. Lock via MCP (Active -> Locked)
        res = call_mcp_tool("aios.session.action", {"session_id": "cross-surface-sess", "action": "lock", "store_path": store_path})
        assert res.get("ok") is True, f"MCP lock failed: {res}"
        assert res.get("report", {}).get("new_state") == "locked", f"Expected locked state: {res}"

        # 6. Verify locked via CLI
        cp = subprocess.run(
            [str(cli_bin), "session", "show", "cross-surface-sess", "--store", store_path, "--json"],
            capture_output=True,
            text=True,
        )
        assert cp.returncode == 0, f"CLI show failed: {cp.stderr}"
        cli_out = json.loads(cp.stdout)
        assert cli_out.get("data", {}).get("status", {}).get("locked") is True, f"CLI did not observe locked: {cli_out}"
        assert cli_out.get("data", {}).get("status", {}).get("state") == "locked", f"CLI state mismatch: {cli_out}"

        # 7. Unlock via CLI (Locked -> Active)
        cp = subprocess.run(
            [str(cli_bin), "session", "unlock", "cross-surface-sess", "--store", store_path, "--json"],
            capture_output=True,
            text=True,
        )
        assert cp.returncode == 0, f"CLI unlock failed: {cp.stderr}"

        # 8. Terminate via MCP: Active -> Terminating -> Terminated
        res1 = call_mcp_tool("aios.session.action", {"session_id": "cross-surface-sess", "action": "terminate", "store_path": store_path})
        assert res1.get("ok") is True, f"MCP terminate failed: {res1}"
        assert res1.get("report", {}).get("new_state") == "terminating", f"Expected terminating state: {res1}"

        res2 = call_mcp_tool("aios.session.action", {"session_id": "cross-surface-sess", "action": "terminate", "store_path": store_path})
        assert res2.get("ok") is True, f"MCP second terminate failed: {res2}"
        assert res2.get("report", {}).get("new_state") == "terminated", f"Expected terminated state: {res2}"

        # 9. Verify terminated via CLI
        cp = subprocess.run(
            [str(cli_bin), "session", "show", "cross-surface-sess", "--store", store_path, "--json"],
            capture_output=True,
            text=True,
        )
        assert cp.returncode == 0, f"CLI show after terminate failed: {cp.stderr}"
        cli_out = json.loads(cp.stdout)
        assert cli_out.get("data", {}).get("status", {}).get("state") == "terminated", f"CLI state mismatch: {cli_out}"

        print("PASS: Cross-surface CLI <-> MCP parity & state sharing")
    finally:
        if os.path.exists(store_path):
            os.unlink(store_path)


def test_session_mcp_hardening():
    # 1. aios.session.validate: oversized payload (>1 MiB)
    res = call_mcp_tool("aios.session.validate", {"spec": "X" * (1024 * 1024 + 10)})
    assert res.get("ok") is False, f"Expected rejection of oversized validate payload: {res}"
    err_str = str(res.get("error", ""))
    assert "1048576 bytes" in err_str or "1 MiB" in err_str, f"Expected 1048576 bytes / 1 MiB message: {res}"

    # 2. aios.session.list: limit out of bounds (0 or >10,000)
    res = call_mcp_tool("aios.session.list", {"limit": 0})
    assert res.get("ok") is False, f"Expected rejection of limit=0: {res}"
    assert "Limit must be between 1 and 10,000" in str(res.get("error", "")), f"Expected limit error: {res}"

    res = call_mcp_tool("aios.session.list", {"limit": 50000})
    assert res.get("ok") is False, f"Expected rejection of limit=50000: {res}"
    assert "Limit must be between 1 and 10,000" in str(res.get("error", "")), f"Expected limit error: {res}"

    # 3. aios.session.list: store_path bounds (>1024 chars or control chars)
    res = call_mcp_tool("aios.session.list", {"store_path": "a" * 1025})
    assert res.get("ok") is False, f"Expected rejection of oversized store_path: {res}"
    assert "1024" in str(res.get("error", "")), f"Expected 1024 error: {res}"

    res = call_mcp_tool("aios.session.list", {"store_path": "bad\x01path"})
    assert res.get("ok") is False, f"Expected rejection of control char store_path: {res}"
    assert "control characters" in str(res.get("error", "")), f"Expected control char error: {res}"

    # 4. aios.session.get: invalid session_id
    res = call_mcp_tool("aios.session.get", {"session_id": "bad/id/traversal"})
    assert res.get("ok") is False, f"Expected rejection of invalid session_id: {res}"
    assert "Invalid session_id" in str(res.get("error", "")), f"Expected Invalid session_id error: {res}"

    # 5. aios.session.get: store_path bounds
    res = call_mcp_tool("aios.session.get", {"session_id": "valid-id", "store_path": "a" * 1025})
    assert res.get("ok") is False, f"Expected rejection of oversized store_path on get: {res}"
    assert "1024" in str(res.get("error", "")), f"Expected 1024 error: {res}"

    # 6. aios.session.action: invalid session_id
    res = call_mcp_tool("aios.session.action", {"session_id": "../evil", "action": "lock"})
    assert res.get("ok") is False, f"Expected rejection of invalid session_id on action: {res}"
    assert "Invalid session_id" in str(res.get("error", "")), f"Expected Invalid session_id error: {res}"

    # 7. aios.session.action: store_path bounds
    res = call_mcp_tool("aios.session.action", {"session_id": "valid-id", "action": "lock", "store_path": "a" * 1025})
    assert res.get("ok") is False, f"Expected rejection of oversized store_path on action: {res}"
    assert "1024" in str(res.get("error", "")), f"Expected 1024 error: {res}"

    # 8. aios.session.create: oversized spec payload (>1 MiB)
    res = call_mcp_tool("aios.session.create", {"spec": "Y" * (1024 * 1024 + 10)})
    assert res.get("ok") is False, f"Expected rejection of oversized create payload: {res}"
    err_str = str(res.get("error", ""))
    assert "1048576 bytes" in err_str or "1 MiB" in err_str, f"Expected 1048576 bytes / 1 MiB error: {res}"

    # 9. aios.session.create: store_path bounds
    dummy_spec = {
        "session_id": "dummy-hard",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "tty",
        "session_class": "user",
        "seat": "seat0",
        "vtnr": 1,
        "display": None,
        "remote_host": None,
        "environment": {}
    }
    res = call_mcp_tool("aios.session.create", {"spec": dummy_spec, "store_path": "a" * 1025})
    assert res.get("ok") is False, f"Expected rejection of oversized store_path on create: {res}"
    assert "1024" in res.get("error", ""), f"Expected 1024 error: {res}"

    print("PASS: MCP session hardening (payload limits, query bounds, ID injection, store path sanitization)")


def main():
    print("=== RUNNING USER SESSION BOOTSTRAP MCP SMOKE TESTS ===")
    test_manifest()
    test_session_validate()
    test_session_list_and_get()
    test_session_action()
    test_session_create_and_persistence()
    test_cross_surface_cli_mcp_parity()
    test_session_mcp_hardening()
    print("\nALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!")
    return 0



if __name__ == "__main__":
    sys.exit(main())
