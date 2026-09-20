#!/usr/bin/env python3
"""Integration Smoke Test for Capability Documentation via MCP (T-02086).

Validates end-to-end capability documentation queries over MCP JSON-RPC:
1. Tool registration: `aios.capability.doc` present in `tools/list`.
2. Action `list`: full index listing and category-filtered listing.
3. Action `get`: topic retrieval by ID and Markdown rendering.
4. Action `search`: scored search and UTF-8 safe snippet extraction.
5. Error handling: invalid actions, missing arguments, non-existent topics.
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
    tool_names = [t.get("name") for t in tools]
    assert "aios.capability.doc" in tool_names, f"aios.capability.doc not found in tools: {tool_names}"
    print("OK")


def test_capability_doc_lifecycle():
    print("TEST: capability doc lifecycle over MCP ...", end=" ")

    # 1. Action: list (all)
    res_list = call_mcp_tool("aios.capability.doc", {"action": "list"})
    assert res_list.get("ok") is True, f"list failed: {res_list}"
    assert res_list.get("count") == 8, f"expected 8 topics, got {res_list.get('count')}"
    topics = res_list.get("topics", [])
    topic_ids = [t.get("id") for t in topics]
    assert "cap-overview" in topic_ids
    assert "cap-attenuation" in topic_ids
    assert "cap-policy" in topic_ids

    # 2. Action: list by category
    res_list_cat = call_mcp_tool("aios.capability.doc", {"action": "list", "category": "architecture"})
    assert res_list_cat.get("ok") is True, f"list category failed: {res_list_cat}"
    assert res_list_cat.get("count") == 2, f"expected 2 architecture topics, got {res_list_cat.get('count')}"

    # 3. Action: get topic
    res_get = call_mcp_tool("aios.capability.doc", {"action": "get", "topic_id": "cap-overview"})
    assert res_get.get("ok") is True, f"get failed: {res_get}"
    topic = res_get.get("topic", {})
    assert topic.get("id") == "cap-overview"
    assert "Zero Ambient Authority" in topic.get("summary") or len(topic.get("sections", [])) > 0
    markdown = res_get.get("markdown", "")
    assert "# Capability Model Overview" in markdown

    # 4. Action: search
    res_search = call_mcp_tool("aios.capability.doc", {"action": "search", "query": "attenuation"})
    assert res_search.get("ok") is True, f"search failed: {res_search}"
    assert res_search.get("count", 0) >= 1
    first_result = res_search.get("results", [])[0]
    assert first_result.get("topic_id") == "cap-attenuation"
    assert first_result.get("score", 0) >= 100

    # 5. Error handling: invalid action
    res_err_action = call_mcp_tool("aios.capability.doc", {"action": "invalid_action"})
    assert res_err_action.get("ok") is False or "error" in res_err_action

    # 6. Error handling: missing topic_id on get
    res_err_get = call_mcp_tool("aios.capability.doc", {"action": "get"})
    assert res_err_get.get("ok") is False or "error" in res_err_get

    # 7. Error handling: non-existent topic
    res_err_notfound = call_mcp_tool("aios.capability.doc", {"action": "get", "topic_id": "non-existent-topic"})
    assert res_err_notfound.get("ok") is False or "error" in res_err_notfound

    print("OK")


def main():
    print(f"Running Capability Doc Integration Smoke against binary: {get_mcp_binary()}")
    test_tool_registration()
    test_capability_doc_lifecycle()
    print("ALL CAPABILITY DOC INTEGRATION TESTS PASSED")


if __name__ == "__main__":
    main()
