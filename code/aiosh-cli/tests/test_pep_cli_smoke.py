#!/usr/bin/env python3
"""CLI Smoke & Invariant Test for PEP Decision Engine (T-02121..T-02125)."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def get_binary_path() -> str:
    candidates = [
        ROOT / "code/aiosh-rust/target/debug/aiosh.exe",
        ROOT / "code/aiosh-rust/target/debug/aiosh",
        ROOT / "target/debug/aiosh.exe",
        ROOT / "target/debug/aiosh",
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    return "aiosh"


def run_aiosh(*args: str) -> subprocess.CompletedProcess:
    bin_path = get_binary_path()
    cmd = [bin_path, *args]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    return res


def parse_json_output(res: subprocess.CompletedProcess) -> dict | list:
    output = res.stdout.strip() or res.stderr.strip()
    return json.loads(output)


def test_pep_help():
    res = run_aiosh("pep", "--help")
    assert res.returncode == 0, f"Unexpected returncode {res.returncode}: {res.stderr}"
    assert "aiosh pep" in res.stdout
    assert "evaluate" in res.stdout
    assert "rule-add" in res.stdout
    assert "rule-list" in res.stdout
    assert "rule-remove" in res.stdout
    assert "status" in res.stdout
    assert "report" in res.stdout
    assert "doc" in res.stdout
    print("PASS: aiosh pep --help")


def test_pep_unknown_subcommand():
    res = run_aiosh("pep", "unknown_cmd")
    assert res.returncode == 2, f"Expected 2, got {res.returncode}"
    print("PASS: aiosh pep unknown_cmd returns 2")


def test_pep_path_hygiene():
    with tempfile.TemporaryDirectory() as tmpdir:
        bad_ext = str(Path(tmpdir) / "pep.txt")
        res = run_aiosh("pep", "status", "--store", bad_ext)
        assert res.returncode == 2, f"Expected 2 for invalid extension, got {res.returncode}"

        traversal = str(Path(tmpdir) / "sub/../pep.json")
        res2 = run_aiosh("pep", "status", "--store", traversal)
        assert res2.returncode == 2, f"Expected 2 for traversal, got {res2.returncode}"
    print("PASS: aiosh pep path hygiene enforcement")


def test_pep_lifecycle():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_policies.json")

        # 1. Initial status empty
        res = run_aiosh("pep", "status", "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["rules_count"] == 0

        # 2. Evaluate without rules -> default deny (exit code 1)
        res = run_aiosh(
            "pep", "evaluate",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/q1.pdf",
            "--action", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1, f"Expected 1 for deny, got {res.returncode}"
        data = parse_json_output(res)
        assert data["code"] == 1
        assert data["data"]["allowed"] is False
        assert data["data"]["effect"] == "deny"

        # 3. Add rule with invalid effect -> fails with 2
        res = run_aiosh(
            "pep", "rule-add",
            "--id", "r1",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/*",
            "--action", "read",
            "--effect", "maybe",
            "--store", store,
            "--json",
        )
        assert res.returncode == 2

        # 4. Add rule with valid effect -> succeeds with 0
        res = run_aiosh(
            "pep", "rule-add",
            "--id", "rule_allow_reports",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/*",
            "--action", "read",
            "--effect", "permit",
            "--desc", "allow reports read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["id"] == "rule_allow_reports"

        # 5. Evaluate matching request -> permit (exit code 0)
        res = run_aiosh(
            "pep", "evaluate",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/q1.pdf",
            "--action", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert data["data"]["allowed"] is True
        assert data["data"]["effect"] == "permit"
        assert data["data"]["matched_rule_id"] == "rule_allow_reports"

        # 6. Evaluate non-matching action -> deny (exit code 1)
        res = run_aiosh(
            "pep", "evaluate",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/q1.pdf",
            "--action", "write",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1
        data = parse_json_output(res)
        assert data["code"] == 1
        assert data["data"]["allowed"] is False

        # 7. List rules
        res = run_aiosh("pep", "rule-list", "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0
        assert len(data["data"]) == 1

        # 8. Remove rule
        res = run_aiosh("pep", "rule-remove", "rule_allow_reports", "--store", store, "--json")
        assert res.returncode == 0
        data = parse_json_output(res)
        assert data["code"] == 0

        # Remove again -> not found (code 1)
        res = run_aiosh("pep", "rule-remove", "rule_allow_reports", "--store", store, "--json")
        assert res.returncode == 1

        # 9. Evaluate after removal -> deny (code 1)
        res = run_aiosh(
            "pep", "evaluate",
            "--subject", "agent:analyst",
            "--resource", "fs:/data/reports/q1.pdf",
            "--action", "read",
            "--store", store,
            "--json",
        )
        assert res.returncode == 1

    print("PASS: aiosh pep lifecycle and evaluation")


def test_pep_security_policy_privilege_boundary():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_privilege.json")

        # 1. Unprivileged caller attempting Permit on sys:* fails with code 2
        res = run_aiosh(
            "pep", "rule-add",
            "--id", "r_sys_permit",
            "--subject", "agent:worker",
            "--resource", "sys:kernel:module",
            "--action", "load",
            "--effect", "permit",
            "--store", store,
            "--json",
        )
        assert res.returncode == 2, f"Expected 2 for unprivileged restricted Permit, got {res.returncode}"
        data = parse_json_output(res)
        assert data["code"] == 2
        assert "POLICY_VIOLATION" in data["error"]["code"]

        # 2. Privileged caller attempting Permit on sys:* succeeds with code 0
        res = run_aiosh(
            "pep", "rule-add",
            "--id", "r_sys_permit",
            "--subject", "agent:worker",
            "--resource", "sys:kernel:module",
            "--action", "load",
            "--effect", "permit",
            "--privileged",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0, f"Expected 0 for privileged Permit, got {res.returncode}: {res.stderr}"

        # 3. Unprivileged caller attempting Deny on sys:* succeeds with code 0
        res = run_aiosh(
            "pep", "rule-add",
            "--id", "r_sys_deny",
            "--subject", "agent:worker",
            "--resource", "sys:kernel:module",
            "--action", "unload",
            "--effect", "deny",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0, f"Expected 0 for unprivileged Deny, got {res.returncode}: {res.stderr}"

    print("PASS: aiosh pep security policy privilege boundary")


def test_pep_report_cli():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_store.json")
        res = run_aiosh("pep", "report", "--store", store, "--json")
        assert res.returncode == 0, f"Expected 0, got {res.returncode}: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        rep = data["data"]
        assert rep["total_rules"] == 0
        assert rep["is_healthy"] is True
        assert rep["capacity_limit"] > 0
        assert rep["capacity_utilization_percent"] == 0
        assert "permit" in rep["rules_by_effect"]

    print("PASS: aiosh pep report CLI integration")


def test_pep_doc_cli():
    # 1. doc list
    res = run_aiosh("pep", "doc", "list", "--json")
    assert res.returncode == 0, f"doc list failed: {res.stderr}"
    data = parse_json_output(res)
    assert data["code"] == 0
    assert data["data"]["count"] >= 6
    topic_ids = {t["id"] for t in data["data"]["topics"]}
    assert "pep-arch" in topic_ids
    assert "pep-algorithms" in topic_ids

    # 2. doc show
    res_show = run_aiosh("pep", "doc", "show", "pep-arch", "--json")
    assert res_show.returncode == 0, f"doc show failed: {res_show.stderr}"
    data_show = parse_json_output(res_show)
    assert data_show["code"] == 0
    assert data_show["data"]["id"] == "pep-arch"
    assert len(data_show["data"]["sections"]) >= 1

    # 3. doc search
    res_search = run_aiosh("pep", "doc", "search", "DenyOverrides", "--json")
    assert res_search.returncode == 0, f"doc search failed: {res_search.stderr}"
    data_search = parse_json_output(res_search)
    assert data_search["code"] == 0
    assert data_search["data"]["count"] >= 1
    assert data_search["data"]["results"][0]["topic_id"] == "pep-algorithms"

    print("PASS: aiosh pep doc CLI integration")


def test_pep_recovery_cli():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_store.json")

        # 1. Validate empty/fresh store -> code 0
        with open(store, "w") as f:
            f.write('{"rules": []}')
        res_val = run_aiosh("pep", "validate", "--store", store, "--json")
        assert res_val.returncode == 0, f"Expected 0, got {res_val.returncode}: {res_val.stderr}"
        d_val = parse_json_output(res_val)
        assert d_val["code"] == 0
        assert d_val["data"]["is_valid"] is True

        # 2. Corrupted store validation -> code 2
        with open(store, "w") as f:
            f.write('{"rules": [{"id": "bad", "target_resource": "fs:/../etc/passwd"}]}')
        res_bad = run_aiosh("pep", "validate", "--store", store, "--json")
        assert res_bad.returncode == 2, f"Expected 2 on bad store, got {res_bad.returncode}"
        d_bad = parse_json_output(res_bad)
        assert d_bad["code"] == 2
        assert d_bad["data"]["is_valid"] is False

        # 3. Salvage recovery -> code 0
        res_rec = run_aiosh("pep", "recover", "--store", store, "--salvage", "--json")
        assert res_rec.returncode == 0, f"Expected 0 on salvage, got {res_rec.returncode}: {res_rec.stderr}"
        d_rec = parse_json_output(res_rec)
        assert d_rec["code"] == 0
        assert d_rec["data"]["success"] is True

        # 4. Store is now valid again
        res_after = run_aiosh("pep", "validate", "--store", store, "--json")
        assert res_after.returncode == 0
        assert parse_json_output(res_after)["data"]["is_valid"] is True

    print("PASS: aiosh pep recovery & validation CLI integration")


def test_pep_grant_cli():
    with tempfile.TemporaryDirectory() as tmpdir:
        grant_store = os.path.join(tmpdir, "pep_grants.json")
        now = "2026-09-22T10:00:00Z"
        test_data = {
            "grants": {
                "grnt-cli-1": {
                    "id": "grnt-cli-1",
                    "parent_grant_id": None,
                    "issuer": "root-admin",
                    "subject": "agent-worker",
                    "scope": {
                        "type": "filesystem",
                        "details": { "path": "/data/test", "recursive": True }
                    },
                    "rights": ["read", "write"],
                    "state": "active",
                    "constraints": {
                        "not_before": None,
                        "expires_at": None,
                        "max_invocations": None,
                        "invocations_used": 0,
                        "max_bytes": None,
                        "bytes_used": 0,
                        "max_delegation_depth": 2
                    },
                    "revocation": None,
                    "metadata": {},
                    "created_at": now,
                    "updated_at": now
                }
            }
        }
        with open(grant_store, "w") as f:
            json.dump(test_data, f)

        # 1. List grants
        res_list = run_aiosh("pep", "grant", "list", "--store", grant_store, "--json")
        assert res_list.returncode == 0, f"Expected 0, got {res_list.returncode}: {res_list.stderr}"
        d_list = parse_json_output(res_list)
        assert d_list["code"] == 0
        assert d_list["data"]["count"] == 1

        # 2. Inspect grant
        res_insp = run_aiosh("pep", "grant", "inspect", "grnt-cli-1", "--store", grant_store, "--json")
        assert res_insp.returncode == 0
        d_insp = parse_json_output(res_insp)
        assert d_insp["code"] == 0
        assert d_insp["data"]["id"] == "grnt-cli-1"
        assert d_insp["data"]["subject"] == "agent-worker"

        # 3. Validate grant
        res_val = run_aiosh("pep", "grant", "validate", "grnt-cli-1", "--subject", "agent-worker", "--right", "read", "--store", grant_store, "--json")
        assert res_val.returncode == 0
        d_val = parse_json_output(res_val)
        assert d_val["code"] == 0
        assert d_val["data"]["valid"] is True

        # 4. Revoke grant
        res_rev = run_aiosh("pep", "grant", "revoke", "grnt-cli-1", "--reason", "Security audit test", "--store", grant_store, "--json")
        assert res_rev.returncode == 0
        d_rev = parse_json_output(res_rev)
        assert d_rev["code"] == 0
        assert d_rev["data"]["revoked_count"] == 1

        # 5. Validate revoked grant fails
        res_val_after = run_aiosh("pep", "grant", "validate", "grnt-cli-1", "--store", grant_store, "--json")
        assert res_val_after.returncode == 1
        d_val_after = parse_json_output(res_val_after)
        assert d_val_after["code"] == 1
        assert d_val_after["data"]["valid"] is False

    print("PASS: aiosh pep grant CLI integration")


if __name__ == "__main__":
    test_pep_help()
    test_pep_unknown_subcommand()
    test_pep_path_hygiene()
    test_pep_lifecycle()
    test_pep_security_policy_privilege_boundary()
    test_pep_report_cli()
    test_pep_doc_cli()
    test_pep_recovery_cli()
    test_pep_grant_cli()
    print("=== All PEP CLI tests passed ===")

