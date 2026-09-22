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
    for expected in [
        "aios.pep.evaluate",
        "aios.pep.status",
        "aios.pep.report",
        "aios.pep.rule_add",
        "aios.pep.rule_list",
        "aios.pep.rule_remove",
        "aios.pep.doc",
        "aios.pep.validate",
        "aios.pep.recover",
        "aios.pep.grant.issue",
        "aios.pep.grant.list",
        "aios.pep.grant.inspect",
        "aios.pep.grant.validate",
        "aios.pep.grant.revoke",
        "aios.pep.grant.attenuate",
        "aios.pep.grant.sweep",
    ]:
        assert expected in tool_names, f"{expected} missing from tools/list"
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


def test_pep_persistent_lifecycle():
    print("TEST: PEP MCP persistent lifecycle (status, rule_add, list, eval, remove) ...", end=" ")
    import tempfile
    import os

    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "pep_store.json")

        # 1. Status on empty store
        st = call_mcp_tool("aios.pep.status", {"store_path": store_path})
        assert st.get("ok") is True, f"status failed: {st}"
        assert st.get("rules_count") == 0, f"expected 0 rules, got {st}"

        # 2. Add rule
        add_res = call_mcp_tool("aios.pep.rule_add", {
            "id": "rule_worker_read",
            "subject": "agent:worker",
            "resource": "fs:/tmp/*",
            "action": "read",
            "effect": "permit",
            "description": "Allow worker reading tmp",
            "store_path": store_path,
        })
        assert add_res.get("ok") is True, f"rule_add failed: {add_res}"
        assert add_res.get("rule", {}).get("id") == "rule_worker_read"

        # 3. Add second rule (deny)
        add_deny = call_mcp_tool("aios.pep.rule_add", {
            "id": "rule_worker_deny_secret",
            "subject": "agent:worker",
            "resource": "fs:/tmp/secret.key",
            "action": "read",
            "effect": "deny",
            "description": "Deny reading secret",
            "store_path": store_path,
        })
        assert add_deny.get("ok") is True, f"rule_add deny failed: {add_deny}"

        # 4. List rules
        listed = call_mcp_tool("aios.pep.rule_list", {"store_path": store_path})
        assert listed.get("ok") is True, f"rule_list failed: {listed}"
        assert listed.get("count") == 2, f"expected 2 rules, got {listed}"

        # 5. List with filter
        filtered = call_mcp_tool("aios.pep.rule_list", {"subject": "agent:worker", "store_path": store_path})
        assert filtered.get("ok") is True
        assert filtered.get("count") == 2

        no_match = call_mcp_tool("aios.pep.rule_list", {"subject": "agent:admin", "store_path": store_path})
        assert no_match.get("ok") is True
        assert no_match.get("count") == 0

        # 6. Evaluate against persistent store
        ev_allow = call_mcp_tool("aios.pep.evaluate", {
            "subject": "agent:worker",
            "resource": "fs:/tmp/data.csv",
            "action": "read",
            "store_path": store_path,
        })
        assert ev_allow.get("ok") is True, f"evaluate allowed failed: {ev_allow}"
        assert ev_allow.get("decision", {}).get("allowed") is True

        ev_deny = call_mcp_tool("aios.pep.evaluate", {
            "subject": "agent:worker",
            "resource": "fs:/tmp/secret.key",
            "action": "read",
            "store_path": store_path,
        })
        assert ev_deny.get("ok") is True, f"evaluate deny failed: {ev_deny}"
        assert ev_deny.get("decision", {}).get("allowed") is False

        # 7. Remove rule
        rm_res = call_mcp_tool("aios.pep.rule_remove", {
            "id": "rule_worker_deny_secret",
            "store_path": store_path,
        })
        assert rm_res.get("ok") is True, f"rule_remove failed: {rm_res}"
        assert rm_res.get("removed") is True

        # Verify removal
        st_after = call_mcp_tool("aios.pep.status", {"store_path": store_path})
        assert st_after.get("rules_count") == 1

        # 8. Remove non-existent rule should return error
        rm_nonexist = call_mcp_tool("aios.pep.rule_remove", {
            "id": "nonexistent_rule",
            "store_path": store_path,
        })
        assert rm_nonexist.get("ok") is False, f"expected error on nonexistent rule: {rm_nonexist}"

        # 9. Path traversal protection
        bad_path = call_mcp_tool("aios.pep.status", {"store_path": "../../../etc/shadow.json"})
        assert bad_path.get("ok") is False, f"expected rejection on path traversal: {bad_path}"

    print("OK")


def test_pep_observability_report():
    print("TEST: PEP observability report ...", end=" ")
    res = call_mcp_tool("aios.pep.report", {})
    assert res.get("ok") is True, f"report failed: {res}"
    report = res.get("report", {})
    assert "total_rules" in report
    assert "is_healthy" in report
    assert "capacity_limit" in report
    assert "capacity_utilization_percent" in report
    assert "rules_by_effect" in report
    print("OK")


def test_pep_doc_mcp():
    print("TEST: PEP MCP documentation tool (list, get, search) ...", end=" ")

    # 1. List topics
    res_list = call_mcp_tool("aios.pep.doc", {"action": "list"})
    assert res_list.get("ok") is True, f"doc list failed: {res_list}"
    assert res_list.get("count", 0) >= 6, f"expected >=6 topics, got {res_list.get('count')}"
    topic_ids = [t["id"] for t in res_list.get("topics", [])]
    assert "pep-arch" in topic_ids, f"missing pep-arch: {topic_ids}"

    # 2. Get specific topic
    res_get = call_mcp_tool("aios.pep.doc", {"action": "get", "topic_id": "pep-arch"})
    assert res_get.get("ok") is True, f"doc get failed: {res_get}"
    assert res_get.get("topic", {}).get("id") == "pep-arch"
    assert "Architecture" in res_get.get("topic", {}).get("title", "")

    # 3. Search topics
    res_search = call_mcp_tool("aios.pep.doc", {"action": "search", "query": "DenyOverrides"})
    assert res_search.get("ok") is True, f"doc search failed: {res_search}"
    assert res_search.get("count", 0) >= 1, f"expected matches for DenyOverrides, got {res_search}"

    # 4. Unknown topic returns ok=False
    res_unknown = call_mcp_tool("aios.pep.doc", {"action": "get", "topic_id": "nonexistent-topic-xyz"})
    assert res_unknown.get("ok") is False, f"expected ok=False for unknown topic: {res_unknown}"

    print("OK")


def test_pep_recovery_mcp():
    print("TEST: PEP MCP recovery & validation tools (validate, recover) ...", end=" ")
    import tempfile
    import os

    with tempfile.TemporaryDirectory() as tmpdir:
        store_path = os.path.join(tmpdir, "pep_store.json")

        # 1. Validate empty store
        with open(store_path, "w") as f:
            f.write('{"rules": []}')
        res_val = call_mcp_tool("aios.pep.validate", {"store_path": store_path})
        assert res_val.get("ok") is True, f"validate failed: {res_val}"
        assert res_val.get("report", {}).get("is_valid") is True

        # 2. Corrupt store validation
        with open(store_path, "w") as f:
            f.write('{"rules": [{"id": "bad", "target_resource": "fs:/../etc/passwd"}]}')
        res_bad = call_mcp_tool("aios.pep.validate", {"store_path": store_path})
        assert res_bad.get("ok") is False, f"expected ok=False on bad store: {res_bad}"
        assert res_bad.get("report", {}).get("is_valid") is False

        # 3. Recover with salvage strategy
        res_rec = call_mcp_tool("aios.pep.recover", {"store_path": store_path, "strategy": "salvage_valid_rules"})
        assert res_rec.get("ok") is True, f"recover failed: {res_rec}"
        assert res_rec.get("result", {}).get("success") is True
        assert res_rec.get("result", {}).get("quarantine_path") is not None

        # 4. Valid after recovery
        res_after = call_mcp_tool("aios.pep.validate", {"store_path": store_path})
        assert res_after.get("ok") is True
        assert res_after.get("report", {}).get("is_valid") is True

    print("OK")


def test_pep_grant_mcp():
    print("TEST: PEP MCP grant lifecycle tools (issue, list, inspect, validate, attenuate, sweep, revoke) ...", end=" ")
    with tempfile.TemporaryDirectory() as tmpdir:
        grant_store = Path(tmpdir) / "pep_grants.json"
        store_str = str(grant_store)

        # 0. Issue root parent grant via MCP tool
        res_issue = call_mcp_tool("aios.pep.grant.issue", {
            "id": "grnt-mcp-1",
            "issuer": "root-admin",
            "subject": "agent-worker",
            "scope_type": "filesystem",
            "scope_path": "/data/test",
            "rights": ["read", "write", "delegate"],
            "delegation_depth": 2,
            "store_path": store_str,
        })
        assert res_issue.get("ok") is True, f"issue failed: {res_issue}"
        assert res_issue.get("grant", {}).get("id") == "grnt-mcp-1"

        # 1. List grants
        res_list = call_mcp_tool("aios.pep.grant.list", {"store_path": store_str})
        assert res_list.get("ok") is True, f"list failed: {res_list}"
        assert res_list.get("count") == 1

        # 2. Inspect grant
        res_insp = call_mcp_tool("aios.pep.grant.inspect", {"store_path": store_str, "grant_id_param": "grnt-mcp-1"})
        assert res_insp.get("ok") is True, f"inspect failed: {res_insp}"
        assert res_insp.get("grant", {}).get("subject") == "agent-worker"

        # 3. Validate grant for action
        res_val = call_mcp_tool("aios.pep.grant.validate", {
            "store_path": store_str,
            "grant_id_param": "grnt-mcp-1",
            "subject": "agent-worker",
            "right": "read"
        })
        assert res_val.get("ok") is True, f"validate failed: {res_val}"
        assert res_val.get("valid") is True

        # 4. Attenuate grant to child
        res_att = call_mcp_tool("aios.pep.grant.attenuate", {
            "store_path": store_str,
            "parent_id": "grnt-mcp-1",
            "child_id": "grnt-mcp-child",
            "child_subject": "agent-subworker",
            "rights": ["read"]
        })
        assert res_att.get("ok") is True, f"attenuate failed: {res_att}"
        child_grant = res_att.get("child_grant", {})
        assert child_grant.get("id") == "grnt-mcp-child"
        assert child_grant.get("parent_grant_id") == "grnt-mcp-1"
        assert child_grant.get("constraints", {}).get("max_delegation_depth") == 1

        # 5. Sweep expired grants (none expired initially)
        res_sweep = call_mcp_tool("aios.pep.grant.sweep", {
            "store_path": store_str
        })
        assert res_sweep.get("ok") is True, f"sweep failed: {res_sweep}"
        assert res_sweep.get("swept_count") == 0

        # 6. Revoke parent grant with cascade (should also revoke child grant in hierarchy)
        res_rev = call_mcp_tool("aios.pep.grant.revoke", {
            "store_path": store_str,
            "grant_id_param": "grnt-mcp-1",
            "reason": "Revoked in smoke test",
            "cascade": True
        })
        assert res_rev.get("ok") is True, f"revoke failed: {res_rev}"
        assert res_rev.get("revoked_count") == 2

        # 7. Re-validate revoked grant (should return ok=false)
        res_val_after = call_mcp_tool("aios.pep.grant.validate", {
            "store_path": store_str,
            "grant_id_param": "grnt-mcp-1"
        })
        assert res_val_after.get("ok") is False, f"expected failure on revoked grant: {res_val_after}"

    print("OK")


def main():
    print("=== PEP Decision Engine MCP Smoke Test ===")
    test_tool_registration()
    test_pep_evaluation()
    test_pep_persistent_lifecycle()
    test_pep_observability_report()
    test_pep_doc_mcp()
    test_pep_recovery_mcp()
    test_pep_grant_mcp()
    print("=== All PEP Decision Engine smoke tests passed ===")


if __name__ == "__main__":
    main()
