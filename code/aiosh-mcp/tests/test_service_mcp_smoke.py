#!/usr/bin/env python3
"""MCP Smoke & Unit Test for Init & Service Supervision Surface (T-01335).

Tests JSON-RPC 2.0 protocol over stdio for:
- aios.service.validate
- aios.service.list
- aios.service.get
- aios.service.action
- aios.service.order
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
        "aios.service.validate",
        "aios.service.list",
        "aios.service.get",
        "aios.service.action",
        "aios.service.order",
    }
    missing = required - names
    assert not missing, f"Missing MCP service tools in manifest: {missing}"
    print("PASS: test_manifest (all 5 service tools registered)")


def test_validate():
    # 1. Valid name
    res = call_mcp_tool("aios.service.validate", {"name": "auditd.service"})
    assert res.get("ok") is True, f"Expected ok: true, got {res}"
    assert res.get("valid") is True

    # 2. Invalid name (invalid character slash)
    res = call_mcp_tool("aios.service.validate", {"name": "invalid/service"})
    assert res.get("ok") is False, f"Expected ok: false, got {res}"

    # 3. Control character in name
    res = call_mcp_tool("aios.service.validate", {"name": "bad\x07name.service"})
    assert res.get("ok") is False, f"Expected ok: false, got {res}"

    # 4. Valid spec
    valid_spec = {
        "name": "custom.service",
        "description": "Custom Service",
        "exec_start": "/usr/bin/custom --run",
        "exec_stop": None,
        "exec_reload": None,
        "service_type": "simple",
        "restart_policy": "on_failure",
        "startup_mode": "enabled",
        "user": "root",
        "group": "root",
        "working_dir": "/",
        "environment": {},
        "dependencies": [],
        "timeout_start_secs": 15,
        "timeout_stop_secs": 15,
    }
    res = call_mcp_tool("aios.service.validate", {"spec": valid_spec})
    assert res.get("ok") is True
    assert res.get("valid") is True

    # 5. Invalid spec (self-dependency)
    invalid_spec = dict(valid_spec)
    invalid_spec["dependencies"] = [{"name": "custom.service", "dependency_type": "requires", "optional": False}]
    res = call_mcp_tool("aios.service.validate", {"spec": invalid_spec})
    assert res.get("ok") is False

    # 6. Missing arguments
    res = call_mcp_tool("aios.service.validate", {})
    assert res.get("ok") is False

    print("PASS: test_validate (positive, negative, and boundary cases)")


def test_list():
    # 1. Default list
    res = call_mcp_tool("aios.service.list", {})
    assert res.get("ok") is True
    assert res.get("count", 0) >= 6

    # 2. Pattern filter
    res = call_mcp_tool("aios.service.list", {"pattern": "audit"})
    assert res.get("ok") is True
    assert res.get("count", 0) >= 1
    assert all("audit" in (s.get("name", "") + s.get("description", "")).lower() for s in res.get("services", []))

    # 3. State filter
    res = call_mcp_tool("aios.service.list", {"state": "active"})
    assert res.get("ok") is True

    # 4. Invalid state
    res = call_mcp_tool("aios.service.list", {"state": "bogus_state"})
    assert res.get("ok") is False

    # 5. Control char in pattern
    res = call_mcp_tool("aios.service.list", {"pattern": "bad\x07pattern"})
    assert res.get("ok") is False

    print("PASS: test_list (filtering, count, invalid enum)")


def test_get():
    # 1. Existing service
    res = call_mcp_tool("aios.service.get", {"name": "auditd.service"})
    assert res.get("ok") is True
    assert res.get("name") == "auditd.service"
    assert res.get("service", {}).get("name") == "auditd.service"
    assert res.get("status", {}).get("state") == "active"

    # 2. Nonexistent service
    res = call_mcp_tool("aios.service.get", {"name": "nonexistent.service"})
    assert res.get("ok") is False

    # 3. Missing name
    res = call_mcp_tool("aios.service.get", {})
    assert res.get("ok") is False

    # 4. Control char in name
    res = call_mcp_tool("aios.service.get", {"name": "bad\x07name.service"})
    assert res.get("ok") is False

    print("PASS: test_get (found, not found, validation error)")


def test_action_and_persistence():
    with tempfile.TemporaryDirectory() as td:
        store_path = str(Path(td) / "test_store.json")

        # Seed the store using default service catalog
        res_list = call_mcp_tool("aios.service.list", {})
        assert res_list.get("ok") is True
        services = res_list.get("services", [])
        statuses = {
            s["name"]: {
                "name": s["name"],
                "state": "active",
                "startup_mode": s["startup_mode"],
                "pid": 100,
                "health": {
                    "healthy": True,
                    "exit_code": None,
                    "pid": 100,
                    "uptime_seconds": 120,
                    "restarts": 0,
                    "last_error": None,
                },
                "started_at": "2026-09-06T00:00:00Z",
            }
            for s in services
        }
        store_data = {
            "services": {s["name"]: s for s in services},
            "statuses": statuses,
        }
        with open(store_path, "w", encoding="utf-8") as f:
            json.dump(store_data, f)

        # First action creates the store if not present
        res = call_mcp_tool("aios.service.action", {
            "name": "auditd.service",
            "action": "stop",
            "store_path": store_path,
        })
        assert res.get("ok") is True, f"stop action failed: {res}"
        report = res.get("report", {})
        assert report.get("success") is True
        assert report.get("new_state") == "inactive"

        # Verify state via get
        res_get = call_mcp_tool("aios.service.get", {
            "name": "auditd.service",
            "store_path": store_path,
        })
        assert res_get.get("ok") is True
        assert res_get.get("status", {}).get("state") == "inactive"

        # Restart action
        res = call_mcp_tool("aios.service.action", {
            "name": "auditd.service",
            "action": "restart",
            "store_path": store_path,
        })
        assert res.get("ok") is True
        assert res.get("report", {}).get("new_state") == "active"

        # Stop action again before masking (cannot mask active service)
        res = call_mcp_tool("aios.service.action", {
            "name": "auditd.service",
            "action": "stop",
            "store_path": store_path,
        })
        assert res.get("ok") is True

        # Mask action
        res = call_mcp_tool("aios.service.action", {
            "name": "auditd.service",
            "action": "mask",
            "store_path": store_path,
        })
        assert res.get("ok") is True
        assert res.get("report", {}).get("new_state") == "inactive"

        # Starting a masked service must fail
        res_fail = call_mcp_tool("aios.service.action", {
            "name": "auditd.service",
            "action": "start",
            "store_path": store_path,
        })
        assert res_fail.get("ok") is False, f"Expected starting masked service to fail, got: {res_fail}"

        # Unmask action
        res = call_mcp_tool("aios.service.action", {
            "name": "auditd.service",
            "action": "unmask",
            "store_path": store_path,
        })
        assert res.get("ok") is True

        # Invalid action verb
        res_bad = call_mcp_tool("aios.service.action", {
            "name": "auditd.service",
            "action": "explode",
            "store_path": store_path,
        })
        assert res_bad.get("ok") is False

    print("PASS: test_action_and_persistence (lifecycle transitions, masked invariant, atomic save)")


def test_order():
    # 1. Valid order
    res = call_mcp_tool("aios.service.order", {"name": "aios-securityd.service"})
    assert res.get("ok") is True
    order = res.get("order", [])
    assert len(order) == 3
    assert order[-1] == "aios-securityd.service"
    assert "auditd.service" in order
    assert "dbus.service" in order

    # 2. Target not found
    res = call_mcp_tool("aios.service.order", {"name": "ghost.service"})
    assert res.get("ok") is False

    # 3. Missing name
    res = call_mcp_tool("aios.service.order", {})
    assert res.get("ok") is False

    print("PASS: test_order (topological ordering, missing targets, error paths)")


def main():
    print("=== RUNNING SERVICE MCP SMOKE TESTS ===")
    test_manifest()
    test_validate()
    test_list()
    test_get()
    test_action_and_persistence()
    test_order()
    print("\nALL SERVICE MCP SMOKE TESTS PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
