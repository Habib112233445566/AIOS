"""Persistent MCP client bridge for the Sprint-2 agent.

The TypeScript agent does not execute host binaries or call Python tool
functions directly. It sends newline-delimited bridge requests here;
this process opens the real `aiosh_mcp.server` over MCP stdio and
forwards `tools/list` and `tools/call` through the protocol.

Bridge protocol (stdout is JSONL only; diagnostics go to stderr):

  request: {"id": 1, "op": "list"}
  response: {"id": 1, "ok": true, "tools": [{"name": ..., "description": ...}]}

  request: {"id": 2, "op": "call", "tool": "pentest.nmap",
            "arguments": {"target": "10.0.0.5", "grant_id": "gr_..."}}
  response: {"id": 2, "ok": true, "tool": "pentest.nmap",
             "result": { ... }, "is_error": false}

Canonical AIOS tool names are mapped to the server's registered
FastMCP names. The server remains authoritative for classifier, PEP,
and audit enforcement; this bridge is transport only.
"""

from __future__ import annotations

import asyncio
import json
import os
import sys
from collections.abc import Mapping
from typing import Any

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


CANONICAL_TO_MCP: dict[str, str] = {
    "aios.fs.read": "aios_fs_read",
    "aios.process.list": "aios_process_list",
    "aios.audit.tail": "aios_audit_tail",
    "aios.audit.verify": "aios_audit_verify",
    "pentest.nmap": "aios_pentest_nmap",
    "pentest.nikto": "aios_pentest_nikto",
    "pentest.sqlmap": "aios_pentest_sqlmap",
    "pentest.tshark": "aios_pentest_tshark",
    "pentest.aircrack-ng": "aios_pentest_aircrack_ng",
}
MCP_TO_CANONICAL = {v: k for k, v in CANONICAL_TO_MCP.items()}


def _jsonable(value: Any) -> Any:
    """Convert MCP/Pydantic values to JSON-safe primitives."""
    if value is None or isinstance(value, (str, int, float, bool)):
        return value
    if isinstance(value, Mapping):
        return {str(k): _jsonable(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [_jsonable(v) for v in value]
    if hasattr(value, "model_dump"):
        return _jsonable(value.model_dump(mode="json"))
    if hasattr(value, "__dict__"):
        return _jsonable(vars(value))
    return str(value)


def _content_to_json(result: Any) -> dict[str, Any]:
    """Flatten CallToolResult into the bridge response.

    FastMCP returns structured content when a tool returns a dict. Older
    MCP versions may expose only TextContent, so both forms are handled.
    """
    dumped = _jsonable(result)
    if not isinstance(dumped, dict):
        return {"value": dumped}
    structured = dumped.get("structuredContent")
    if structured is None:
        structured = dumped.get("structured_content")
    if isinstance(structured, dict):
        # FastMCP commonly nests the return under `result` or emits the
        # dict directly. Preserve the direct dict when it is available.
        candidate = structured.get("result", structured)
        if isinstance(candidate, dict):
            return candidate
    content = dumped.get("content")
    if isinstance(content, list):
        for item in content:
            if not isinstance(item, dict):
                continue
            text = item.get("text")
            if isinstance(text, str):
                try:
                    parsed = json.loads(text)
                    if isinstance(parsed, dict):
                        return parsed
                except json.JSONDecodeError:
                    return {"ok": False, "text": text}
    return dumped


async def _serve() -> int:
    # The bridge is launched from aiosh-cli, so force the MCP package root
    # into the child environment. The server itself is the actual MCP
    # peer; no direct Python import of a tool function occurs here.
    root = os.environ.get("AIOSH_MCP_ROOT")
    if not root:
        root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    env = dict(os.environ)
    old_pythonpath = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = root + (os.pathsep + old_pythonpath if old_pythonpath else "")
    params = StdioServerParameters(
        command=sys.executable,
        args=["-m", "aiosh_mcp.server"],
        env=env,
        cwd=root,
    )

    async with stdio_client(params, errlog=sys.stderr) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            listed = await session.list_tools()
            known = {
                item.name for item in listed.tools
                if item.name in MCP_TO_CANONICAL
            }
            print(json.dumps({
                "event": "ready",
                "protocol": "mcp-stdio",
                "tools": sorted(MCP_TO_CANONICAL[n] for n in known),
            }, separators=(",", ":")), flush=True)

            for raw in sys.stdin:
                raw = raw.strip()
                if not raw:
                    continue
                request: Any = None
                try:
                    request = json.loads(raw)
                    response = await _handle(session, known, request)
                except Exception as exc:  # bridge stays alive per request
                    response = {
                        "id": request.get("id") if isinstance(request, dict) else None,
                        "ok": False,
                        "error": f"bridge error: {exc}",
                    }
                print(json.dumps(_jsonable(response), separators=(",", ":")),
                      flush=True)
    return 0


async def _handle(session: ClientSession, known: set[str], request: dict[str, Any]) -> dict[str, Any]:
    req_id = request.get("id")
    op = request.get("op")
    if op == "list":
        return {"id": req_id, "ok": True,
                "tools": sorted(MCP_TO_CANONICAL[n] for n in known)}
    if op != "call":
        return {"id": req_id, "ok": False, "error": f"unknown op: {op!r}"}

    canonical = request.get("tool")
    if not isinstance(canonical, str) or canonical not in CANONICAL_TO_MCP:
        return {"id": req_id, "ok": False, "error": "unknown canonical tool"}
    mcp_name = CANONICAL_TO_MCP[canonical]
    if mcp_name not in known:
        return {"id": req_id, "ok": False,
                "error": f"tool not advertised by MCP server: {canonical}"}
    arguments = request.get("arguments", {})
    if not isinstance(arguments, dict):
        return {"id": req_id, "ok": False,
                "error": "arguments must be a JSON object"}
    result = await session.call_tool(mcp_name, arguments)
    flattened = _content_to_json(result)
    return {
        "id": req_id,
        "ok": True,
        "tool": canonical,
        "mcp_tool": mcp_name,
        "is_error": bool(getattr(result, "isError", False)
                          or getattr(result, "is_error", False)),
        "result": flattened,
    }


def main() -> int:
    try:
        return asyncio.run(_serve())
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
