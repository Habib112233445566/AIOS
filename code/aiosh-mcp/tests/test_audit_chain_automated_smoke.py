#!/usr/bin/env python3
"""Automated Cross-Surface Integration Smoke Test for Audit Chain Extensions (Sub-Epic 6, T-02355..T-02356).

Tests automated end-to-end query filtering, DAG ancestry tracing, signature validation,
and configuration reflection against the compiled aiosh-mcp binary over JSON-RPC.

Run standalone:
    python code/aiosh-mcp/tests/test_audit_chain_automated_smoke.py
"""

from __future__ import annotations

import json
import os
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


def test_automated_config_and_bounds():
    """Verify automated configuration retrieval and bounds."""
    cfg = call_mcp_tool("aios.audit.config", {})
    assert cfg.get("ok") is True, f"Failed to get config: {cfg}"
    assert cfg.get("version") == "1.0.0"
    assert cfg.get("max_query_limit") == 50
    assert cfg.get("default_lineage_depth") == 16
    print("PASS: test_automated_config_and_bounds")


def test_automated_query_stress():
    """Perform rapid queries with multiple filters."""
    for limit in [1, 5, 20]:
        res = call_mcp_tool("aios.audit.query", {"limit": limit})
        assert res.get("ok") is True
        rows = res.get("matches", res.get("events", res.get("rows", [])))
        assert len(rows) <= limit
    print("PASS: test_automated_query_stress")


def test_automated_ancestry_and_signatures():
    """Trace ancestry and verify signatures across available events."""
    query = call_mcp_tool("aios.audit.query", {"limit": 3})
    rows = query.get("matches", query.get("events", query.get("rows", [])))
    for r in rows:
        h = r.get("row_hash", r.get("hash"))
        if h:
            anc = call_mcp_tool("aios.audit.ancestry", {"hash": h, "depth": 5})
            assert anc.get("ok") is True
            sig = call_mcp_tool("aios.audit.sign_verify", {"hash": h})
            assert sig.get("ok") is True
    print("PASS: test_automated_ancestry_and_signatures")


if __name__ == "__main__":
    test_automated_config_and_bounds()
    test_automated_query_stress()
    test_automated_ancestry_and_signatures()
    print("=== All Automated MCP Audit Chain Smoke Tests PASSED ===")
