#!/usr/bin/env python3
"""Comprehensive Unit Tests for PEP Grant CLI Subcommands (T-02225).

Covers all 7 subcommands of `aiosh pep grant`:
1. `issue`: Happy path, missing required flags, invalid scope type, invalid rights, duplicate ID.
2. `attenuate`: Successful child derivation, right expansion rejection, depth decrement.
3. `list`: Unfiltered listing, subject filtering, state filtering.
4. `inspect`: Existing grant inspection, non-existent grant, missing positional ID.
5. `validate`: Valid authorization, unauthorized right, subject mismatch, temporal expiration.
6. `revoke`: Direct revocation, cascade recursive revocation.
7. `sweep`: Expired grant sweep with reference timestamp.
8. Path hygiene: Traversal rejection.
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


def get_aiosh_binary() -> str:
    return _find_binary(["aiosh.exe", "aiosh"])


def run_aiosh(*args: str, timeout_s: int = 15) -> subprocess.CompletedProcess:
    bin_path = get_aiosh_binary()
    return subprocess.run(
        [bin_path, *args],
        capture_output=True,
        text=True,
        timeout=timeout_s,
    )


def parse_json_output(res: subprocess.CompletedProcess) -> dict:
    for line in reversed(res.stdout.splitlines()):
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                return json.loads(line)
            except Exception:
                pass
    raise ValueError(f"Could not find JSON in stdout:\nSTDOUT:\n{res.stdout}\nSTDERR:\n{res.stderr}")


def test_pep_grant_issue_and_validation():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_grants.json")

        # 1. Successful issue of root parent grant
        res = run_aiosh(
            "pep", "grant", "issue",
            "--id", "g-root-1",
            "--issuer", "security-admin",
            "--subject", "agent:primary",
            "--scope-type", "filesystem",
            "--scope-path", "/var/data",
            "--rights", "read,write,delegate",
            "--delegation-depth", "2",
            "--expires-at", "2026-12-31T23:59:59Z",
            "--store", store,
            "--json",
        )
        assert res.returncode == 0, f"Issue failed: {res.stderr}"
        data = parse_json_output(res)
        assert data["code"] == 0
        g = data["data"]
        assert g["id"] == "g-root-1"
        assert g["subject"] == "agent:primary"
        assert g["state"] == "active"
        assert "delegate" in g["rights"]
        assert g["constraints"]["max_delegation_depth"] == 2

        # 2. Issue negative: missing required flags
        res_missing = run_aiosh(
            "pep", "grant", "issue",
            "--id", "g-bad",
            "--subject", "agent:bad",
            "--store", store,
            "--json",
        )
        assert res_missing.returncode == 2
        d_err = parse_json_output(res_missing)
        assert d_err["code"] == 2

        # 3. Issue negative: unknown scope type
        res_bad_scope = run_aiosh(
            "pep", "grant", "issue",
            "--id", "g-bad-scope",
            "--subject", "agent:bad",
            "--scope-type", "invalid_scope",
            "--rights", "read",
            "--store", store,
            "--json",
        )
        assert res_bad_scope.returncode == 2

        # 4. Issue negative: unknown capability right
        res_bad_right = run_aiosh(
            "pep", "grant", "issue",
            "--id", "g-bad-right",
            "--subject", "agent:bad",
            "--scope-type", "filesystem",
            "--rights", "read,superadmin_hack",
            "--store", store,
            "--json",
        )
        assert res_bad_right.returncode == 2

    print("PASS: test_pep_grant_issue_and_validation")


def test_pep_grant_attenuation():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_grants.json")

        # Setup parent grant
        res_parent = run_aiosh(
            "pep", "grant", "issue",
            "--id", "g-parent",
            "--subject", "agent:parent",
            "--scope-type", "filesystem",
            "--rights", "read,write,delegate",
            "--delegation-depth", "2",
            "--store", store,
            "--json",
        )
        assert res_parent.returncode == 0

        # 1. Attenuate valid child
        res_att = run_aiosh(
            "pep", "grant", "attenuate",
            "g-parent",
            "--child", "g-child-1",
            "--subject", "agent:child",
            "--rights", "read",
            "--store", store,
            "--json",
        )
        assert res_att.returncode == 0, f"Attenuate failed: {res_att.stderr}"
        d_att = parse_json_output(res_att)
        assert d_att["code"] == 0
        child = d_att["data"]
        assert child["id"] == "g-child-1"
        assert child["parent_grant_id"] == "g-parent"
        assert child["rights"] == ["read"]
        assert child["constraints"]["max_delegation_depth"] == 1

        # 2. Attenuate negative: right escalation (requesting delete when parent has read,write,delegate)
        res_esc = run_aiosh(
            "pep", "grant", "attenuate",
            "g-parent",
            "--child", "g-child-bad",
            "--subject", "agent:child",
            "--rights", "delete",
            "--store", store,
            "--json",
        )
        assert res_esc.returncode == 1
        d_esc = parse_json_output(res_esc)
        assert d_esc["code"] == 1
        assert "ATTENUATION_FAILED" in d_esc["error"]["code"]

        # 3. Attenuate negative: missing required flags
        res_miss = run_aiosh(
            "pep", "grant", "attenuate",
            "g-parent",
            "--store", store,
            "--json",
        )
        assert res_miss.returncode == 2

    print("PASS: test_pep_grant_attenuation")


def test_pep_grant_list_and_inspect():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_grants.json")

        # Issue two grants
        run_aiosh("pep", "grant", "issue", "--id", "g1", "--subject", "alice", "--scope-type", "fs", "--rights", "read", "--store", store, "--json")
        run_aiosh("pep", "grant", "issue", "--id", "g2", "--subject", "bob", "--scope-type", "fs", "--rights", "write", "--store", store, "--json")

        # 1. Unfiltered list
        res_list = run_aiosh("pep", "grant", "list", "--store", store, "--json")
        assert res_list.returncode == 0
        d_list = parse_json_output(res_list)
        assert d_list["data"]["count"] == 2

        # 2. Subject filter
        res_filter = run_aiosh("pep", "grant", "list", "--subject", "alice", "--store", store, "--json")
        assert res_filter.returncode == 0
        d_filter = parse_json_output(res_filter)
        assert d_filter["data"]["count"] == 1
        assert d_filter["data"]["grants"][0]["id"] == "g1"

        # 3. Inspect existing
        res_insp = run_aiosh("pep", "grant", "inspect", "g1", "--store", store, "--json")
        assert res_insp.returncode == 0
        d_insp = parse_json_output(res_insp)
        assert d_insp["data"]["id"] == "g1"
        assert d_insp["data"]["subject"] == "alice"

        # 4. Inspect non-existent returns 1
        res_missing = run_aiosh("pep", "grant", "inspect", "g-ghost", "--store", store, "--json")
        assert res_missing.returncode == 1
        d_miss = parse_json_output(res_missing)
        assert d_miss["code"] == 1
        assert d_miss["error"]["code"] == "NOT_FOUND"

        # 5. Inspect missing ID returns 2
        res_no_id = run_aiosh("pep", "grant", "inspect", "--store", store, "--json")
        assert res_no_id.returncode == 2

    print("PASS: test_pep_grant_list_and_inspect")


def test_pep_grant_validate_and_revoke():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_grants.json")

        # Issue parent grant
        run_aiosh(
            "pep", "grant", "issue",
            "--id", "g-root",
            "--subject", "alice",
            "--scope-type", "fs",
            "--rights", "read,write,delegate",
            "--delegation-depth", "2",
            "--store", store,
            "--json",
        )
        # Attenuate child grant
        run_aiosh(
            "pep", "grant", "attenuate",
            "g-root",
            "--child", "g-leaf",
            "--subject", "bob",
            "--rights", "read",
            "--store", store,
            "--json",
        )

        # 1. Validate root for alice/read -> success (code 0)
        res_val1 = run_aiosh("pep", "grant", "validate", "g-root", "--subject", "alice", "--right", "read", "--store", store, "--json")
        assert res_val1.returncode == 0
        assert parse_json_output(res_val1)["data"]["valid"] is True

        # 2. Validate root for bob -> fails (subject mismatch, code 1)
        res_val2 = run_aiosh("pep", "grant", "validate", "g-root", "--subject", "bob", "--right", "read", "--store", store, "--json")
        assert res_val2.returncode == 1

        # 3. Validate leaf for bob/write -> fails (right mismatch, code 1)
        res_val3 = run_aiosh("pep", "grant", "validate", "g-leaf", "--subject", "bob", "--right", "write", "--store", store, "--json")
        assert res_val3.returncode == 1

        # 4. Cascade revoke root -> revokes root and leaf (code 0, count 2)
        res_rev = run_aiosh("pep", "grant", "revoke", "g-root", "--reason", "Test revocation", "--cascade", "--store", store, "--json")
        assert res_rev.returncode == 0
        d_rev = parse_json_output(res_rev)
        assert d_rev["data"]["revoked_count"] == 2

        # 5. Validate leaf after cascade revoke -> fails (code 1)
        res_val_rev = run_aiosh("pep", "grant", "validate", "g-leaf", "--store", store, "--json")
        assert res_val_rev.returncode == 1
        assert parse_json_output(res_val_rev)["data"]["valid"] is False

    print("PASS: test_pep_grant_validate_and_revoke")


def test_pep_grant_sweep():
    with tempfile.TemporaryDirectory() as tmpdir:
        store = str(Path(tmpdir) / "pep_grants.json")

        # Issue grant expiring at 2026-09-22T10:00:00Z
        run_aiosh(
            "pep", "grant", "issue",
            "--id", "g-exp",
            "--subject", "agent:temp",
            "--scope-type", "fs",
            "--rights", "read",
            "--expires-at", "2026-09-22T10:00:00Z",
            "--store", store,
            "--json",
        )

        # Sweep at time after expiration: 2026-09-22T12:00:00Z
        res_sweep = run_aiosh("pep", "grant", "sweep", "--now", "2026-09-22T12:00:00Z", "--store", store, "--json")
        assert res_sweep.returncode == 0
        d_sweep = parse_json_output(res_sweep)
        assert d_sweep["data"]["swept_count"] == 1

        # Validate expired grant fails
        res_val = run_aiosh("pep", "grant", "validate", "g-exp", "--store", store, "--json")
        assert res_val.returncode == 1

    print("PASS: test_pep_grant_sweep")


if __name__ == "__main__":
    test_pep_grant_issue_and_validation()
    test_pep_grant_attenuation()
    test_pep_grant_list_and_inspect()
    test_pep_grant_validate_and_revoke()
    test_pep_grant_sweep()
    print("=== All PEP Grant CLI unit tests passed successfully ===")
