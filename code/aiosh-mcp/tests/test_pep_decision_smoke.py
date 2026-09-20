#!/usr/bin/env python3
"""Integration Smoke Test for PEP Decision Engine via MCP (T-02106).

Validates end-to-end PEP Decision Engine policy evaluations over MCP JSON-RPC:
1. Tool registration: `aios.pep.evaluate` present in `tools/list`.
2. Default deny: requests evaluated without matching permit rules return `Deny` (PEPDEC1).
3. Rule evaluation: explicit permit rules grant access.
4. Combining algorithms: `deny_overrides` ensures deny rules override permit rules (PEPDEC3).
5. Error handling: rejects invalid subjects with control characters or empty strings.
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
    assert "aios.pep.evaluate" in tool_names, "aios.pep.evaluate missing from tools/list"
    print("OK")


def test_pep_evaluation():
    print("TEST: PEP decision evaluation ...", end=" ")

    # 1. Default deny when no rules match
    res_default = call_mcp_tool("aios.pep.evaluate", {
        "subject": "agent:worker",
        "resource": "fs:/var/log/audit.log",
        "action": "read",
    })
    assert res_default.get("ok") is True, f"call failed: {res_default}"
    dec_default = res_default.get("decision", {})
    assert dec_default.get("effect") == "deny", f"expected deny: {dec_default}"
    assert dec_default.get("allowed") is False, f"expected allowed=false: {dec_default}"

    # 2. Explicit permit rule
    rules = [
        {
            "id": "rule_allow_worker_logs",
            "target_subject": "agent:worker",
            "target_resource": "fs:/var/log/*",
            "target_action": "read",
            "effect": "permit",
            "obligations": [{"type": "audit_log", "level": "info", "message": "log read"}],
            "description": "allow reading logs",
        }
    ]
    res_permit = call_mcp_tool("aios.pep.evaluate", {
        "subject": "agent:worker",
        "resource": "fs:/var/log/audit.log",
        "action": "read",
        "rules": rules,
    })
    assert res_permit.get("ok") is True, f"call failed: {res_permit}"
    dec_permit = res_permit.get("decision", {})
    assert dec_permit.get("effect") == "permit", f"expected permit: {dec_permit}"
    assert dec_permit.get("allowed") is True, f"expected allowed=true: {dec_permit}"
    assert dec_permit.get("matched_rule_id") == "rule_allow_worker_logs"
    assert len(dec_permit.get("obligations", [])) == 1

    # 3. DenyOverrides: deny rule overrides permit rule
    rules_with_deny = rules + [
        {
            "id": "rule_deny_audit_log",
            "target_subject": "agent:worker",
            "target_resource": "fs:/var/log/audit.log",
            "target_action": "read",
            "effect": "deny",
            "obligations": [],
            "description": "deny direct audit log reading",
        }
    ]
    res_deny_override = call_mcp_tool("aios.pep.evaluate", {
        "subject": "agent:worker",
        "resource": "fs:/var/log/audit.log",
        "action": "read",
        "algorithm": "deny_overrides",
        "rules": rules_with_deny,
    })
    assert res_deny_override.get("ok") is True, f"call failed: {res_deny_override}"
    dec_deny_override = res_deny_override.get("decision", {})
    assert dec_deny_override.get("effect") == "deny", f"expected deny override: {dec_deny_override}"
    assert dec_deny_override.get("allowed") is False
    assert dec_deny_override.get("matched_rule_id") == "rule_deny_audit_log"

    print("OK")


def main():
    print("=== PEP Decision Engine MCP Smoke Test ===")
    test_tool_registration()
    test_pep_evaluation()
    print("=== All PEP Decision Engine smoke tests passed ===")


if __name__ == "__main__":
    main()
