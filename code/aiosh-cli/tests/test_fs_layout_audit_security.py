#!/usr/bin/env python3
"""Audit-Emission & Terminal-Injection Security Proof for Filesystem Layout (T-01527).

Proves, through the real `aiosh` binary, the two findings raised by the T-01527 security
review of the Filesystem Layout CLI surface:

  F-1 (fail-open audit gap, ADR-0035 §F-2): the `probe`, `diff` and unknown-subcommand failure
      paths must each emit exactly one audit row before exiting, like every sibling surface.

  F-2 (CWE-150 escape-sequence injection): layout specs may be authored outside the operator's
      trust boundary, so spec-derived text rendered to the terminal must not be able to emit
      control characters — while `--json` output and the on-disk store must stay faithful.

Observable evidence is the real audit ring (`$AIOSH_HOME/audit.db`, table `audit_ring`) and the
process exit status / stdout / stderr of the binary.

Run standalone:
    python code/aiosh-cli/tests/test_fs_layout_audit_security.py
"""

from __future__ import annotations

import json
import os
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

AUDIT_HOME = tempfile.mkdtemp(prefix="aios_fslayout_audit_")
ESC = "\x1b"
BEL = "\x07"


# ---------------------------------------------------------------------------
# Harness
# ---------------------------------------------------------------------------

def get_binary_path() -> str:
    for base in ("code/aiosh-rust/target/debug", "target/debug"):
        for name in ("aiosh.exe", "aiosh"):
            candidate = ROOT / base / name
            if candidate.exists():
                return str(candidate)
    return "aiosh"


def run_cli(*args: str, expect: int = 0, context: str = "") -> subprocess.CompletedProcess:
    env = dict(os.environ)
    # Isolate the audit ring so rows can be attributed to this suite's invocations.
    env["AIOSH_HOME"] = AUDIT_HOME
    cp = subprocess.run(
        [get_binary_path(), *args], capture_output=True, text=True, timeout=60, env=env
    )
    assert cp.returncode == expect, (
        f"{context or ' '.join(args)}: expected exit {expect}, got {cp.returncode}\n"
        f"stdout: {cp.stdout.strip()}\nstderr: {cp.stderr.strip()}"
    )
    return cp


def cli_json(*args: str, expect: int = 0, context: str = ""):
    return json.loads(run_cli(*args, expect=expect, context=context).stdout.strip())


def _db() -> Path:
    return Path(AUDIT_HOME) / "audit.db"


def audit_rows() -> list[dict]:
    """Every audit ring row, oldest first. Empty before the first invocation creates the ring."""
    db = _db()
    if not db.exists():
        return []
    con = sqlite3.connect(str(db))
    try:
        cur = con.execute(
            "SELECT id, tool, command, outcome, target, outcome_detail "
            "FROM audit_ring ORDER BY id"
        )
        return [
            {
                "id": r[0],
                "tool": r[1],
                "command": r[2],
                "outcome": r[3],
                "target": r[4],
                "outcome_detail": r[5],
            }
            for r in cur.fetchall()
        ]
    finally:
        con.close()


def max_audit_id() -> int:
    rows = audit_rows()
    return rows[-1]["id"] if rows else 0


def rows_since(mark: int) -> list[dict]:
    return [r for r in audit_rows() if r["id"] > mark]


def fs_layout_rows(mark: int) -> list[dict]:
    return [r for r in rows_since(mark) if r["tool"] == "fs_layout"]


# ---------------------------------------------------------------------------
# F-1: audit emission on every execution path
# ---------------------------------------------------------------------------

def test_success_path_emits_exactly_one_row() -> None:
    mark = max_audit_id()
    run_cli("layout", "list", "--json", context="list")
    assert _db().exists(), f"invocation must create the audit ring at {_db()}"
    rows = fs_layout_rows(mark)
    assert len(rows) == 1, f"expected exactly 1 audit row, got {rows}"
    assert rows[0]["command"] == "list"
    assert rows[0]["outcome"] == "success"
    print("PASS: F-1 successful path emits exactly one audit row")


def test_probe_resolve_failure_is_audited() -> None:
    """Regression for the flagged fail-open gap: probe resolve failure wrote no row."""
    mark = max_audit_id()
    run_cli("layout", "probe", "no-such-layout-xyz", "--json", expect=1,
            context="probe unknown layout")
    rows = fs_layout_rows(mark)
    assert len(rows) == 1, f"probe resolve failure must emit exactly 1 row, got {rows}"
    assert rows[0]["command"] == "probe"
    assert rows[0]["outcome"] == "failure"
    assert rows[0]["outcome_detail"] == "Failed to resolve layout for probe"
    print("PASS: F-1 probe resolve failure is audited")


def test_probe_invalid_bytes_is_audited() -> None:
    mark = max_audit_id()
    run_cli("layout", "probe", "--bytes", "notanumber", "--json", expect=2,
            context="probe invalid bytes")
    rows = fs_layout_rows(mark)
    assert len(rows) == 1, f"probe argument error must emit exactly 1 row, got {rows}"
    assert rows[0]["command"] == "probe"
    assert rows[0]["outcome"] == "failure"
    print("PASS: F-1 probe invalid --bytes is audited")


def test_diff_failure_paths_are_audited() -> None:
    """Regression for the flagged fail-open gap: diff resolve failure wrote no row."""
    # (a) unknown layout identifiers.
    mark = max_audit_id()
    run_cli("layout", "diff", "no-such-a", "no-such-b", "--json", expect=1, context="diff unknown")
    rows = fs_layout_rows(mark)
    assert len(rows) == 1, f"diff failure must emit exactly 1 row, got {rows}"
    assert rows[0]["command"] == "diff"
    assert rows[0]["outcome"] == "failure"
    assert rows[0]["outcome_detail"] == "Failed to compute layout diff"

    # (b) an unreadable store fails during load — also previously unaudited.
    with tempfile.TemporaryDirectory() as td:
        corrupt = Path(td) / "corrupt.json"
        corrupt.write_text("NOT A JSON STORE", encoding="utf-8")
        mark = max_audit_id()
        run_cli("layout", "diff", "--store", str(corrupt), "--json", expect=1,
                context="diff corrupt store")
        rows = fs_layout_rows(mark)
        assert len(rows) == 1, f"diff store load failure must emit 1 row, got {rows}"
        assert rows[0]["command"] == "diff"
        assert rows[0]["outcome_detail"] == "Failed to load layout store for diff"

    print("PASS: F-1 diff failure paths (diff + store load) are audited")


def test_unknown_subcommand_is_audited() -> None:
    mark = max_audit_id()
    run_cli("layout", "bogus_subcommand", "--json", expect=2, context="unknown subcommand")
    rows = fs_layout_rows(mark)
    assert len(rows) == 1, f"unknown subcommand must emit exactly 1 row, got {rows}"
    assert rows[0]["command"] == "unknown"
    assert rows[0]["outcome"] == "failure"
    print("PASS: F-1 unknown subcommand is audited")


def test_every_failure_path_emits_exactly_one_row() -> None:
    """No path may double-log: a representative sweep must be exactly 1 row each."""
    cases = [
        (("layout", "show", "no-such-layout", "--json"), 1),
        (("layout", "validate", "--spec", "{not json", "--json"), 1),
        (("layout", "fstab", "no-such-layout", "--json"), 1),
        (("layout", "register", "--json"), 2),
        (("layout", "set-active", "--json"), 2),
        (("layout", "remove", "--json"), 2),
        (("layout", "import-fstab", "--json"), 2),
        (("layout", "list", "--store", "bad\x01path", "--json"), 2),
    ]
    for argv, expect in cases:
        mark = max_audit_id()
        run_cli(*argv, expect=expect, context=" ".join(argv))
        rows = fs_layout_rows(mark)
        assert len(rows) == 1, f"{argv} emitted {len(rows)} rows (expected exactly 1): {rows}"
    print(f"PASS: F-1 all {len(cases)} sampled failure paths emit exactly one audit row")


# ---------------------------------------------------------------------------
# F-2: terminal escape-sequence injection
# ---------------------------------------------------------------------------

def _malicious_store() -> str:
    """Registers a layout whose free-text fields carry terminal escape sequences."""
    spec = cli_json("layout", "show", "aios-uefi-standard-v1", "--json",
                    context="show base")["data"]
    spec["id"] = "inject-v1"
    spec["name"] = f"Inject{ESC}[31mPWNED{ESC}[0m"
    spec["description"] = f"desc{ESC}]52;c;aGFjaw=={BEL}"
    spec["created_at"] = f"20260101{ESC}[2J"
    spec["partitions"][0]["label"] = f"EFI{ESC}[1m"

    td = tempfile.mkdtemp(prefix="aios_fslayout_inject_")
    store = str(Path(td) / "layouts.json")
    spec_file = Path(td) / "spec.json"
    spec_file.write_text(json.dumps(spec), encoding="utf-8")
    run_cli("layout", "register", "--spec", str(spec_file), "--store", store, "--json",
            context="register malicious spec")
    run_cli("layout", "set-active", "inject-v1", "--store", store, "--json",
            context="activate malicious layout")
    return store


def test_spec_derived_fields_cannot_inject_escapes() -> None:
    store = _malicious_store()

    # Human-readable rendering must never carry raw control characters.
    show_out = run_cli("layout", "show", "inject-v1", "--store", store, context="show").stdout
    assert ESC not in show_out, f"show leaked an ESC sequence: {show_out!r}"
    assert "PWNED" in show_out, "content should be neutralized, not dropped"

    list_out = run_cli("layout", "list", "--store", store, context="list").stdout
    assert ESC not in list_out, f"list leaked an ESC sequence: {list_out!r}"

    fstab_out = run_cli("layout", "fstab", "--store", store, context="fstab").stdout
    assert ESC not in fstab_out, f"fstab leaked an ESC sequence (created_at): {fstab_out!r}"

    # ...while the machine-readable contract stays byte-faithful (serde escapes it correctly).
    payload = cli_json("layout", "show", "inject-v1", "--store", store, "--json")
    assert ESC in payload["data"]["name"], "JSON output must not be lossily sanitized"
    assert BEL in payload["data"]["description"]

    # ...and the canonical store keeps the raw data: sanitization is a rendering concern only.
    raw = json.loads(Path(store).read_text(encoding="utf-8"))
    assert ESC in raw["layouts"]["inject-v1"]["name"]

    print("PASS: F-2 spec-derived fields neutralized on render, faithful in JSON and store")


def test_argv_echoes_are_neutralized() -> None:
    store = _malicious_store()
    evil = f"{ESC}[31mEVIL{ESC}[0m"

    # A rejected id is echoed back inside a core error message on the human path.
    cp = run_cli("layout", "remove", evil, "--store", store, expect=1, context="remove evil id")
    assert ESC not in cp.stderr, f"remove leaked an ESC sequence: {cp.stderr!r}"

    cp = run_cli("layout", "set-active", evil, "--store", store, expect=1,
                 context="set-active evil id")
    assert ESC not in cp.stderr, f"set-active leaked an ESC sequence: {cp.stderr!r}"

    cp = run_cli("layout", "bogus", evil, expect=2, context="unknown subcommand echo")
    assert ESC not in cp.stderr, f"unknown-subcommand echo leaked an ESC sequence: {cp.stderr!r}"

    print("PASS: F-2 argv-derived echoes neutralized on the human path")


def main() -> int:
    print("=== RUNNING FILESYSTEM LAYOUT AUDIT & INJECTION SECURITY PROOF ===")
    test_success_path_emits_exactly_one_row()
    test_probe_resolve_failure_is_audited()
    test_probe_invalid_bytes_is_audited()
    test_diff_failure_paths_are_audited()
    test_unknown_subcommand_is_audited()
    test_every_failure_path_emits_exactly_one_row()
    test_spec_derived_fields_cannot_inject_escapes()
    test_argv_echoes_are_neutralized()
    print("\nALL FILESYSTEM LAYOUT AUDIT & INJECTION SECURITY PROOFS PASSED!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
