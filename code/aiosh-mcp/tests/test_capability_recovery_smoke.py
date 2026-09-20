#!/usr/bin/env python3
"""Integration Smoke Test for Capability Recovery and Validation via MCP (T-02096).

Validates end-to-end capability store recovery and validation over MCP JSON-RPC:
1. Tool registration: `aios.capability.recover` and `aios.capability.validate` in `tools/list`.
2. Clean recovery: non-existent store creates fresh default store.
3. Validation: validates store invariants CAPREC1..CAPREC4.
4. Corrupted store recovery: non-destructive quarantine to .bak.<timestamp> and recovery.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
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
        print(f"FAIL: aiosh-mcp timed out after {timeout_s}s")
        sys.exit(1)
    finally:
        if p.poll() is None:
            p.kill()
            p.wait()

    if p.returncode != 0:
        print(f"FAIL: aiosh-mcp returned {p.returncode}")
        sys.exit(1)
    try:
        return json.loads(stdout.strip())
    except Exception as e:
        print(f"FAIL: invalid JSON from aiosh-mcp: {e}")
        print(stdout)
        sys.exit(1)


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


def test_tool_registration():
    print("TEST: tool registration via tools/list ...", end=" ")
    tools = list_mcp_tools()
    tool_names = {t["name"] for t in tools}
    assert "aios.capability.recover" in tool_names, "aios.capability.recover missing from tools/list"
    assert "aios.capability.validate" in tool_names, "aios.capability.validate missing from tools/list"
    print("OK")


def test_recovery_and_validation():
    print("TEST: capability recovery and validation ...", end=" ")
    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "capability_store.json")

        # 1. Recover non-existent store (creates default)
        rec_res = call_mcp_tool("aios.capability.recover", {"store_path": store_path})
        assert rec_res.get("ok") is True, f"recover failed: {rec_res}"
        assert "CreatedDefaultFresh" in rec_res.get("action", ""), f"unexpected action: {rec_res}"
        assert os.path.exists(store_path), "store file was not created"

        # 2. Validate clean store
        val_res = call_mcp_tool("aios.capability.validate", {"store_path": store_path})
        assert val_res.get("ok") is True, f"validate failed: {val_res}"
        data = val_res.get("data", {})
        assert data.get("healthy") is True, f"store not healthy: {data}"
        assert data.get("total_capabilities") == 0, f"expected 0 caps: {data}"

        # 3. Corrupt store and recover (quarantine)
        with open(store_path, "w", encoding="utf-8") as f:
            f.write("{ invalid json content !!!")

        corrupt_rec_res = call_mcp_tool("aios.capability.recover", {"store_path": store_path})
        assert corrupt_rec_res.get("ok") is True, f"corrupt recover failed: {corrupt_rec_res}"
        action = corrupt_rec_res.get("action", "")
        assert "RecoveredFromBackup" in action, f"expected quarantine backup action: {corrupt_rec_res}"

        # Verify backup was created
        bak_files = [f for f in os.listdir(tmpdir) if f.startswith("capability_store.json.bak.")]
        assert len(bak_files) >= 1, f"no backup files found in {tmpdir}: {os.listdir(tmpdir)}"
        with open(os.path.join(tmpdir, bak_files[0]), "r", encoding="utf-8") as bf:
            assert "{ invalid json content !!!" in bf.read(), "backup content mismatch"

    print("OK")


def main():
    print("=== Capability Recovery & Validation MCP Smoke Test ===")
    test_tool_registration()
    test_recovery_and_validation()
    print("=== All Capability Recovery smoke tests passed ===")


if __name__ == "__main__":
    main()
