#!/usr/bin/env python3
"""Comprehensive Unit and Integration Smoke Tests for Secret MCP Tools (T-02636).

Covers all 5 Secrets Handling tools over MCP JSON-RPC 2.0 stdio:
1. `aios.secret.store`: Storing secret with metadata, scope, and payload.
2. `aios.secret.get`: Default masked retrieval vs explicit `expose: true` retrieval.
3. `aios.secret.list`: Safe metadata listing without payload disclosure.
4. `aios.secret.rotate`: Versioned payload rotation.
5. `aios.secret.revoke`: Secret revocation and denial of post-revocation access.
6. `traversal_defense`: Rejection of path traversal attempts in custom store paths.
"""

from __future__ import annotations

import json
import os
import shutil
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


def run_mcp(payload: dict, env: dict | None = None, timeout_s: int = 30) -> dict:
    run_env = os.environ.copy()
    if env:
        run_env.update(env)
    p = subprocess.Popen(
        [get_mcp_binary()],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        env=run_env,
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


def call_mcp_tool(tool_name: str, arguments: dict | None = None, env: dict | None = None, timeout_s: int = 30) -> dict:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments or {}},
    }
    resp = run_mcp(payload, env=env, timeout_s=timeout_s)
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


def list_mcp_tools(env: dict | None = None, timeout_s: int = 30) -> list[dict]:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {},
    }
    resp = run_mcp(payload, env=env, timeout_s=timeout_s)
    return resp.get("result", {}).get("tools", [])


def test_mcp_secret_tool_registration():
    print("Testing secret tool registration...")
    tools = list_mcp_tools()
    tool_names = {t["name"] for t in tools}
    for expected in (
        "aios.secret.store",
        "aios.secret.get",
        "aios.secret.list",
        "aios.secret.rotate",
        "aios.secret.revoke",
    ):
        assert expected in tool_names, f"Missing MCP tool {expected}"


def test_mcp_secret_lifecycle():
    print("Testing secret MCP lifecycle...")
    tmp_dir = tempfile.mkdtemp(prefix="aiosh_mcp_sec_py_")
    store_file = os.path.join(tmp_dir, "vault.json")
    try:
        # 1. Path traversal rejection
        res = call_mcp_tool("aios.secret.list", {"store_path": "../evil.json"})
        assert res.get("ok") is False or "ERR_SECRET_PATH_TRAVERSAL" in str(res)

        # 2. Store validation error
        res = call_mcp_tool("aios.secret.store", {"name": "No ID", "kind": "api_key", "store_path": store_file})
        assert res.get("ok") is False or "ERR_SECRET_INVALID_INPUT" in str(res)

        # 3. Store valid secret
        res = call_mcp_tool("aios.secret.store", {
            "id": "py_mcp_api_token",
            "name": "Python MCP Test Token",
            "kind": "api_key",
            "value": "super_secret_mcp_token_9999",
            "store_path": store_file,
        })
        assert res.get("ok") is True, f"Store failed: {res}"
        assert res["metadata"]["id"] == "py_mcp_api_token"

        # 4. Get masked secret
        res = call_mcp_tool("aios.secret.get", {
            "id": "py_mcp_api_token",
            "store_path": store_file,
        })
        assert res.get("ok") is True, f"Get masked failed: {res}"
        assert res.get("exposed") is False
        assert "super_secret_mcp_token_9999" not in res.get("value", "")
        assert "..." in res.get("value", "") or "********" in res.get("value", "")

        # 5. Get exposed plaintext secret
        res = call_mcp_tool("aios.secret.get", {
            "id": "py_mcp_api_token",
            "expose": True,
            "store_path": store_file,
        })
        assert res.get("ok") is True, f"Get exposed failed: {res}"
        assert res.get("exposed") is True
        assert res.get("value") == "super_secret_mcp_token_9999"

        # 6. Get non-existent secret
        res = call_mcp_tool("aios.secret.get", {
            "id": "non_existent_key",
            "store_path": store_file,
        })
        assert res.get("ok") is False

        # 7. List secrets
        res = call_mcp_tool("aios.secret.list", {"store_path": store_file})
        assert res.get("ok") is True, f"List failed: {res}"
        assert res.get("count") == 1
        assert "super_secret_mcp_token_9999" not in str(res)

        # 8. Rotate secret
        res = call_mcp_tool("aios.secret.rotate", {
            "id": "py_mcp_api_token",
            "value": "rotated_mcp_token_8888",
            "store_path": store_file,
        })
        assert res.get("ok") is True, f"Rotate failed: {res}"
        assert res.get("version") == 2

        # Verify rotated payload with expose
        res = call_mcp_tool("aios.secret.get", {
            "id": "py_mcp_api_token",
            "expose": True,
            "store_path": store_file,
        })
        assert res.get("value") == "rotated_mcp_token_8888"

        # 9. Revoke secret
        res = call_mcp_tool("aios.secret.revoke", {
            "id": "py_mcp_api_token",
            "store_path": store_file,
        })
        assert res.get("ok") is True, f"Revoke failed: {res}"
        assert res.get("state") == "revoked"

        # 10. Access to revoked secret rejected
        res = call_mcp_tool("aios.secret.get", {
            "id": "py_mcp_api_token",
            "store_path": store_file,
        })
        assert res.get("ok") is False

    finally:
        shutil.rmtree(tmp_dir, ignore_errors=True)


def main():
    print("Running Secrets Handling MCP smoke tests...")
    test_mcp_secret_tool_registration()
    test_mcp_secret_lifecycle()
    print("All Secrets Handling MCP smoke tests PASSED!")


if __name__ == "__main__":
    main()
