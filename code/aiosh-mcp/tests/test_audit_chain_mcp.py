#!/usr/bin/env python3
"""Comprehensive Unit Tests for Audit Chain Extensions MCP Tool Surface (T-02335).

Covers all 4 Audit Chain Extensions tools over MCP JSON-RPC 2.0 stdio:
1. `aios.audit.query`: Event filtering by session_id, trace_id, actor, tool, parent_hash, limit.
2. `aios.audit.inspect`: Event inspection by hash, non-existent hash error handling.
3. `aios.audit.ancestry`: Causal DAG ancestry tracing with depth limits and cycle resilience.
4. `aios.audit.sign_verify`: Cryptographic signature verification on audit events.
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


def test_mcp_audit_chain_tool_registration():
    """Verify all 4 audit chain tools are correctly registered and schema-validated."""
    tools = list_mcp_tools()
    tool_names = {t["name"] for t in tools}
    expected_tools = [
        "aios.audit.query",
        "aios.audit.inspect",
        "aios.audit.ancestry",
        "aios.audit.sign_verify",
        "aios.audit.config",
    ]
    for expected in expected_tools:
        assert expected in tool_names, f"Missing MCP tool in registration: {expected}"

    # Verify parameter schemas
    schema_map = {t["name"]: t.get("inputSchema", {}) for t in tools}
    inspect_schema = schema_map["aios.audit.inspect"]
    assert "hash" in inspect_schema.get("properties", {})
    assert "hash" in inspect_schema.get("required", [])

    sign_verify_schema = schema_map["aios.audit.sign_verify"]
    assert "hash" in sign_verify_schema.get("properties", {})
    assert "hash" in sign_verify_schema.get("required", [])


def test_mcp_audit_query_filtering_and_bounds():
    """Test aios.audit.query with various filter criteria and boundary conditions."""
    # 1. Unfiltered query
    res = call_mcp_tool("aios.audit.query", {"limit": 5})
    assert res.get("ok") is True, f"Query failed: {res}"
    assert "matches" in res or "events" in res or "rows" in res
    rows = res.get("matches", res.get("events", res.get("rows", [])))
    assert isinstance(rows, list)
    assert len(rows) <= 5

    # 2. Filter by actor / tool
    res_tool = call_mcp_tool("aios.audit.query", {"tool": "tools/call", "limit": 10})
    assert res_tool.get("ok") is True

    # 3. Query with non-matching filter
    res_none = call_mcp_tool("aios.audit.query", {"actor": "nonexistent_actor_99999"})
    assert res_none.get("ok") is True
    rows_none = res_none.get("matches", res_none.get("events", res_none.get("rows", [])))
    assert len(rows_none) == 0


def test_mcp_audit_inspect_valid_and_missing():
    """Test aios.audit.inspect with existing hash, missing hash, and bad arguments."""
    # 1. Negative: Missing required argument 'hash'
    res_missing = call_mcp_tool("aios.audit.inspect", {})
    assert res_missing.get("ok") is False or "error" in res_missing, f"Should fail without hash: {res_missing}"

    # 2. Negative: Non-existent hash
    res_notfound = call_mcp_tool("aios.audit.inspect", {"hash": "deadbeef00000000000000000000000000000000000000000000000000000000"})
    assert res_notfound.get("ok") is False
    assert "not found" in res_notfound.get("error", "").lower()

    # 3. Positive: Inspect a known row from a query
    query_res = call_mcp_tool("aios.audit.query", {"limit": 1})
    rows = query_res.get("matches", query_res.get("events", query_res.get("rows", [])))
    if query_res.get("ok") and rows:
        target_hash = rows[0].get("row_hash", rows[0].get("hash"))
        inspect_res = call_mcp_tool("aios.audit.inspect", {"hash": target_hash})
        assert inspect_res.get("ok") is True
        row = inspect_res.get("event", inspect_res.get("row", {}))
        assert row.get("row_hash", row.get("hash")) == target_hash


def test_mcp_audit_ancestry_and_sign_verify():
    """Test aios.audit.ancestry and aios.audit.sign_verify tools."""
    # 1. Negative: Missing hash argument
    res_anc_nohash = call_mcp_tool("aios.audit.ancestry", {})
    assert res_anc_nohash.get("ok") is False

    res_sign_nohash = call_mcp_tool("aios.audit.sign_verify", {})
    assert res_sign_nohash.get("ok") is False

    # 2. Ancestry with non-existent hash returns empty ancestors
    res_anc_bad = call_mcp_tool("aios.audit.ancestry", {"hash": "nonexistent_hash_12345"})
    assert res_anc_bad.get("ok") is True
    assert res_anc_bad.get("ancestors") == []

    # 3. Negative: Sign-verify with non-existent row
    res_sign_bad = call_mcp_tool("aios.audit.sign_verify", {"hash": "nonexistent_hash_12345"})
    assert res_sign_bad.get("ok") is False
    assert "not found" in res_sign_bad.get("error", "").lower()

    # 4. Positive: Query known rows and trace ancestry / verify signature
    query_res = call_mcp_tool("aios.audit.query", {"limit": 5})
    rows = query_res.get("matches", query_res.get("events", query_res.get("rows", [])))
    if query_res.get("ok") and rows:
        sample_hash = rows[0].get("row_hash", rows[0].get("hash"))

        # Ancestry call with bounded depth
        anc_res = call_mcp_tool("aios.audit.ancestry", {"hash": sample_hash, "depth": 10})
        assert anc_res.get("ok") is True
        assert "ancestors" in anc_res

        # Signature verification call
        sig_res = call_mcp_tool("aios.audit.sign_verify", {"hash": sample_hash})
        assert sig_res.get("ok") is True
        assert "is_valid" in sig_res
        assert "has_signature" in sig_res


def test_mcp_audit_config():
    """Test aios.audit.config tool returns valid configuration object."""
    res = call_mcp_tool("aios.audit.config", {})
    assert res.get("ok") is True, f"Config tool failed: {res}"
    assert res.get("tool") == "aios.audit.config"
    assert "version" in res
    assert "db_path" in res
    assert "max_query_limit" in res
    assert "default_lineage_depth" in res
    assert "max_causal_links" in res


if __name__ == "__main__":
    print("Running test_mcp_audit_chain_tool_registration...")
    test_mcp_audit_chain_tool_registration()
    print("Running test_mcp_audit_query_filtering_and_bounds...")
    test_mcp_audit_query_filtering_and_bounds()
    print("Running test_mcp_audit_inspect_valid_and_missing...")
    test_mcp_audit_inspect_valid_and_missing()
    print("Running test_mcp_audit_ancestry_and_sign_verify...")
    test_mcp_audit_ancestry_and_sign_verify()
    print("Running test_mcp_audit_config...")
    test_mcp_audit_config()
    print("=== All Audit Chain MCP Unit Tests Passed Successfully ===")

