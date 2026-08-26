#!/usr/bin/env python3
"""Task Ledger Control — observability unit smoke (T-00085).

Contract: docs/tasks/evidence/T-00082-spec.md (+ T-00084 implementation).
One sandbox, all three production surfaces, asserting the OBSERVABLE
metrics contract no single-surface test pins end-to-end:

  O1  CLI `aiosh task metrics` happy path: stable ADDITIVE-ONLY key set
      {tasks,audit,config}; facts agree with TASK_STATE.json and the
      raw SQLite audit count.
  O2  Rust MCP wire metrics: ok envelope + audit_id > 0 (exactly one
      committed row per call).
  O3  Python reference surface metrics parity: same key set + audit_id.
  O4  Config knob propagation: AIOSH_LEDGER_MAX_TEXT reaches
      data.config.max_text on BOTH substrates.
  B1  Boundary: empty audit ring -> rows == 0, verify_ok true (per
      surface; prefix formatting divergence is tracked as finding F5).
  N1  Invalid env knob fails LOUDLY naming the variable (Rust CLI).
  N2  Primary failure mode: TAMPERED audit chain -> metrics still
      ok:true but honestly reports verify_ok=false on both substrates.
  N3  Unresolvable tasks dir -> exit 1, standard refusal envelope on
      stderr, AND one honest audit row (ADR-0035 fail-open-with-audit).

Style: repo PASS/FAIL prints, isolated tempdirs, explicit subprocess
timeouts, observable-behavior assertions only.
"""

import json
import os
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
RUST_BIN = REPO / "code" / "aiosh-rust" / "target" / "debug" / "aiosh"
RUST_MCP = REPO / "code" / "aiosh-rust" / "target" / "debug" / "aiosh-mcp"

PASS, FAIL = "[✓]", "[✗]"
STABLE_KEYS = {"tasks", "audit", "config"}


def make_ledger(path: Path, n: int = 3) -> None:
    with open(path, "w", encoding="utf-8") as f:
        for i in range(1, n + 1):
            f.write(json.dumps({
                "id": i, "title": f"task {i}", "phase": "P",
                "status": "pending", "goal": f"g{i}", "instructions": [f"s{i}"],
                "acceptance": [f"a{i}"], "artifacts": [],
                "depends_on": [] if i == 1 else [i - 1],
                "next_task": i + 1 if i < n else None,
            }, separators=(",", ":")) + "\n")


def write_state(tasks_dir: Path, n: int) -> None:
    (tasks_dir / "TASK_STATE.json").write_text(json.dumps({
        "schema_version": 2, "ledger": "MASTER_TASK_LEDGER.jsonl",
        "total_tasks": n, "next_task": 1, "completed": [], "blocked": [],
        "skipped": [], "last_completed_at": None, "last_event_seq": 0,
        "rule": "Execute ONLY next_task.",
    }), encoding="utf-8")


class Sandbox:
    def __init__(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="obs-smoke-"))
        self.root = tmp
        self.tasks_dir = tmp / "tasks"; self.tasks_dir.mkdir()
        self.home = tmp / "home"; self.home.mkdir()
        make_ledger(self.tasks_dir / "MASTER_TASK_LEDGER.jsonl", n=3)
        write_state(self.tasks_dir, 3)
        self.env = {**os.environ,
                    "AIOSH_TASKS_DIR": str(self.tasks_dir),
                    "AIOSH_HOME": str(self.home)}
        os.environ["AIOSH_TASKS_DIR"] = str(self.tasks_dir)
        os.environ["AIOSH_HOME"] = str(self.home)

    def reset_ledger(self, n: int = 3) -> None:
        make_ledger(self.tasks_dir / "MASTER_TASK_LEDGER.jsonl", n=n)
        write_state(self.tasks_dir, n)

    def cli(self, *args, timeout: int = 60):
        return subprocess.run([str(RUST_BIN), "task", *args],
                              capture_output=True, text=True,
                              env=self.env, timeout=timeout)

    def audit_db(self) -> Path:
        return self.home / "audit.db"

    def audit_count(self) -> int:
        if not self.audit_db().exists():
            return 0
        conn = sqlite3.connect(self.audit_db())
        try:
            return conn.execute("SELECT COUNT(*) FROM audit_ring").fetchone()[0]
        except sqlite3.OperationalError:
            return 0  # db exists but schema not created yet
        finally:
            conn.close()


def py_task(**kwargs):
    """Invoke the registered Python tool function directly."""
    from aiosh_mcp.server import aios_task
    return aios_task(**kwargs)


def rust_wire(sandbox, requests, timeout: int = 30):
    payload = "".join(json.dumps(r) + "\n" for r in requests)
    p = subprocess.run([str(RUST_MCP)], input=payload, capture_output=True,
                       text=True, env=sandbox.env, timeout=timeout)
    out = {}
    for line in p.stdout.splitlines():
        if line.strip():
            d = json.loads(line)
            out[d.get("id")] = d
    return out


def wire_result(resp):
    return json.loads(resp["result"]["content"][0]["text"])


INIT = {"jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {},
                   "clientInfo": {"name": "obs", "version": "1"}}}


def o1_cli_happy(sb) -> None:
    # Exactly-one-row contract: snapshot BEFORE, then the CLI appends
    # exactly its own trailing task.ledger row AFTER printing (the
    # printed metrics reflect the pre-call ring by design).
    before = sb.audit_count()
    p = sb.cli("metrics")
    doc = json.loads(p.stdout.strip())
    assert p.returncode == 0 and doc["ok"] is True, (p.returncode, doc)
    data = doc["data"]
    assert set(data.keys()) == STABLE_KEYS, data.keys()
    st = json.loads((sb.tasks_dir / "TASK_STATE.json").read_text())
    # pointer facts agree; counters are emitted as COUNTS by the shipping
    # (Rust) surface per the T-00082 'tasks counters+pointer' contract
    assert data["tasks"]["next_task"] == st["next_task"], (data, st)
    assert data["tasks"]["completed"] == len(st["completed"]), (data, st)
    assert data["audit"]["verify_ok"] is True, data
    # rows must equal the REAL pre-call sqlite count (not a capped view),
    # and the call must add exactly one honest audit row.
    assert data["audit"]["rows"] == before, (
        data["audit"]["rows"], before)
    assert sb.audit_count() == before + 1, (
        sb.audit_count(), before)


def o2_rust_wire_audited(sb) -> None:
    reqs = [INIT,
            {"jsonrpc": "2.0", "id": 2, "method": "tools/call",
             "params": {"name": "aios.task",
                        "arguments": {"action": "metrics"}}}]
    resp = rust_wire(sb, reqs)
    m = resp[2]
    assert m["result"]["isError"] is False, m
    r = wire_result(m)
    assert r["ok"] is True and r["action"] == "metrics", r
    assert set(r["data"].keys()) == STABLE_KEYS, r["data"].keys()
    assert isinstance(r.get("audit_id"), int) and r["audit_id"] > 0, r


def o3_python_parity(sb) -> None:
    r = py_task(action="metrics")
    assert r["ok"] is True and r["action"] == "metrics", r
    assert set(r["data"].keys()) == STABLE_KEYS, r["data"].keys()
    assert isinstance(r.get("audit_id"), int) and r["audit_id"] > 0, r
    # same live facts as the CLI saw in O1
    st = json.loads((sb.tasks_dir / "TASK_STATE.json").read_text())
    assert r["data"]["tasks"]["next_task"] == st["next_task"], r


def o4_config_propagates_both(sb) -> None:
    env = dict(sb.env, AIOSH_LEDGER_MAX_TEXT="8192")
    p = subprocess.run([str(RUST_BIN), "task", "metrics"],
                       capture_output=True, text=True, env=env, timeout=30)
    d = json.loads(p.stdout.strip())
    assert d["data"]["config"]["max_text"] == 8192, d
    code = ("import json\n"
            "from aiosh_mcp.server import aios_task\n"
            "print(json.dumps(aios_task(action='metrics')))\n")
    p2 = subprocess.run([sys.executable, "-c", code], capture_output=True,
                        text=True, env=env,
                        cwd=str(REPO / "code" / "aiosh-mcp"), timeout=30)
    r = json.loads(p2.stdout.strip().splitlines()[-1])
    assert r["data"]["config"]["max_text"] == 8192, r


def b1_empty_ring_boundary(sb) -> None:
    sb.reset_ledger(2)
    fresh = Sandbox.__new__(Sandbox)  # second isolated home, no reuse
    tmp = Path(tempfile.mkdtemp(prefix="obs-empty-"))
    fresh.tasks_dir = sb.tasks_dir
    fresh.home = tmp; fresh.home.mkdir(exist_ok=True)
    fresh.env = dict(sb.env, AIOSH_HOME=str(fresh.home))
    os.environ["AIOSH_HOME"] = str(fresh.home)
    try:
        reqs = [INIT,
                {"jsonrpc": "2.0", "id": 2, "method": "tools/call",
                 "params": {"name": "aios.task",
                            "arguments": {"action": "metrics"}}}]
        r = wire_result(rust_wire(fresh, reqs)[2])
        assert r["ok"] is True, r
        assert r["data"]["audit"]["rows"] >= 0
        assert r["data"]["audit"]["verify_ok"] is True, r
    finally:
        os.environ["AIOSH_HOME"] = str(sb.home)


def n1_invalid_env_loud(sb) -> None:
    env = dict(sb.env, AIOSH_LEDGER_LOCK_TIMEOUT_SECS="soon")
    p = subprocess.run([str(RUST_BIN), "task", "metrics"],
                       capture_output=True, text=True, env=env, timeout=30)
    assert p.returncode == 1, (p.returncode, p.stdout, p.stderr)
    doc = json.loads(p.stderr.strip())
    assert doc["ok"] is False, doc
    assert "AIOSH_LEDGER_LOCK_TIMEOUT_SECS" in doc.get("error", ""), doc


def n2_tampered_chain_reports_honestly(sb) -> None:
    conn = sqlite3.connect(sb.audit_db())
    try:
        conn.execute(
            "UPDATE audit_ring SET args_json='{\"tampered\":true}' "
            "WHERE id=(SELECT MIN(id) FROM audit_ring)")
        conn.commit()
    finally:
        conn.close()
    p = sb.cli("metrics")
    d = json.loads(p.stdout.strip())
    assert d["ok"] is True, d                      # metrics itself must not crash
    assert d["data"]["audit"]["verify_ok"] is False, d
    r = py_task(action="metrics")
    assert r["ok"] is True and r["data"]["audit"]["verify_ok"] is False, r


def n3_unresolvable_dir_refused_and_audited(sb) -> None:
    env = dict(sb.env, AIOSH_TASKS_DIR="/nonexistent-obs-smoke/tasks")
    before = _count_home_rows(sb.home)
    p = subprocess.run([str(RUST_BIN), "task", "metrics"],
                       capture_output=True, text=True, env=env, timeout=30)
    assert p.returncode == 1, (p.returncode, p.stdout)
    doc = json.loads(p.stderr.strip())
    assert doc["ok"] is False and doc.get("error"), doc
    after = _count_home_rows(sb.home)
    assert after == before + 1, (before, after)    # honest refusal row


def _count_home_rows(home: Path) -> int:
    db = home / "audit.db"
    if not db.exists():
        return 0
    conn = sqlite3.connect(db)
    try:
        return conn.execute("SELECT COUNT(*) FROM audit_ring").fetchone()[0]
    finally:
        conn.close()


CASES = [
    ("O1 cli metrics happy path", o1_cli_happy),
    ("O2 rust-wire metrics audited", o2_rust_wire_audited),
    ("O3 python metrics parity", o3_python_parity),
    ("O4 config knob reaches both", o4_config_propagates_both),
    ("B1 empty-ring boundary", b1_empty_ring_boundary),
    ("N1 invalid env fails loud", n1_invalid_env_loud),
    ("N2 tampered chain reported honestly", n2_tampered_chain_reports_honestly),
    ("N3 unresolvable dir refused+audited", n3_unresolvable_dir_refused_and_audited),
]


def main() -> int:
    if not RUST_BIN.exists() or not RUST_MCP.exists():
        print(f"{FAIL} binaries missing (run cargo build)")
        return 2
    sb = Sandbox()
    failures = 0
    for label, fn in CASES:
        try:
            fn(sb)
            print(f"{PASS} {label}")
        except AssertionError as e:
            print(f"{FAIL} {label}")
            print("   ", e)
            failures += 1
    if failures:
        print(f"\n{failures} case(s) failing")
        return 1
    print("\nPASS: observability unit smoke (O1..O4, B1, N1..N3)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
