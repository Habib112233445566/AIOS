"""Task Ledger Control — validate_state unit tests (T-00105).

Standalone test for the recovery & validation component
(`tools/task_ledger.py::validate_state`, contract:
docs/tasks/evidence/T-00102-spec.md). Follows the repo smoke-test style
(PASS/FAIL markers, non-zero exit on failure). Runs entirely in a temp
directory via AIOSH_TASKS_DIR — never mutates real docs/tasks state.

Coverage (spec §4 check ids):
  V1  valid input: clean sandbox -> consistent=true, exact key set
  V2  primary failure mode: state-vs-events drift -> fatal + fields,
      report-only (state file byte-identical after run)
  V3  event_seq integrity: renumbered seq -> fatal with detail
  V4  pointer_range: replayed pointer on a currently-blocked id -> fatal
  V5  boundary: empty event log on a fresh ledger -> consistent=true
  V6  boundary: end-of-ledger None pointer stays consistent
  V7  invalid input: missing state file -> FileNotFoundError (loud),
      corrupt event line -> ValueError; no partial findings invented
  V8  evidence warnings: missing referenced path + orphan stub file,
      both warning-only (never flip `consistent`)
"""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
MODULE_PATH = HERE / "task_ledger.py"

PASS = "[✓]"
FAIL = "[✗]"

CHECK_KEYS = {"state_vs_events", "event_seq", "pointer_range", "evidence"}


def load_module(env_home: str):
    """Import task_ledger with AIOSH_TASKS_DIR pointed at the sandbox."""
    spec = importlib.util.spec_from_file_location("task_ledger_v_under_test",
                                                  MODULE_PATH)
    module = importlib.util.module_from_spec(spec)
    old = os.environ.get("AIOSH_TASKS_DIR")
    os.environ["AIOSH_TASKS_DIR"] = env_home
    try:
        spec.loader.exec_module(module)
    finally:
        if old is None:
            os.environ.pop("AIOSH_TASKS_DIR", None)
        else:
            os.environ["AIOSH_TASKS_DIR"] = old
    module.DOCS_TASKS = env_home
    module.LEDGER_PATH = os.path.join(env_home, "MASTER_TASK_LEDGER.jsonl")
    module.STATE_PATH = os.path.join(env_home, "TASK_STATE.json")
    module.EVENTS_PATH = os.path.join(env_home, "COMPLETIONS.jsonl")
    module.LOCK_PATH = os.path.join(env_home, ".TASK_STATE.lock")
    module.EVIDENCE_DIR = os.path.join(env_home, "evidence")
    return module


def make_ledger(path: Path, n: int = 3) -> None:
    with open(path, "w", encoding="utf-8") as f:
        for i in range(1, n + 1):
            rec = {
                "id": i,
                "title": f"task {i}",
                "phase": "Phase 0 — Test",
                "status": "pending",
                "goal": f"goal {i}",
                "instructions": [f"step {i}"],
                "acceptance": [f"accept {i}"],
                "artifacts": [f"evidence/T-{i:05d}.md"],
                "depends_on": [] if i == 1 else [i - 1],
                "next_task": i + 1 if i < n else None,
            }
            f.write(json.dumps(rec, ensure_ascii=False,
                               separators=(",", ":")) + "\n")


def make_fresh_state(path: Path, total: int) -> None:
    with open(path, "w", encoding="utf-8") as f:
        json.dump({
            "schema_version": 2,
            "ledger": "MASTER_TASK_LEDGER.jsonl",
            "total_tasks": total,
            "next_task": 1,
            "completed": [],
            "blocked": [],
            "skipped": [],
            "last_completed_at": None,
            "last_event_seq": 0,
        }, f)


def read_state(tl, path=None):
    return json.loads(open(path or tl.STATE_PATH, encoding="utf-8").read())


def main() -> int:
    tmp = Path(tempfile.mkdtemp(prefix="task-validate-unit-"))
    tl = load_module(str(tmp))
    make_ledger(tmp / "MASTER_TASK_LEDGER.jsonl", n=3)
    make_fresh_state(tmp / "TASK_STATE.json", total=3)

    # ---- V1 — clean sandbox validates consistent with the exact key set
    r = tl.validate_state()
    ok = (r["ok"] is True and r["consistent"] is True
          and set(r["checks"].keys()) == CHECK_KEYS
          and all(r["checks"][k]["status"] == "ok"
                  for k in ("state_vs_events", "event_seq", "pointer_range"))
          and r["replay"]["next_task"] == 1 and r["live"]["next_task"] == 1)
    print(f"{PASS if ok else FAIL} V1 clean repo consistent, key set exact")
    if not ok:
        print(json.dumps(r, indent=2))
        return 1

    # Seed two honest completions so later cases have substance.
    tl.complete_task(1, note="v")
    tl.complete_task(2, note="v")

    # ---- V2 — drift: hand-tampered pointer; report-only proof
    before_bytes = open(tl.STATE_PATH, "rb").read()
    st = read_state(tl)
    st["next_task"] = 1            # diverges from replay pointer 3
    st["completed"] = [1]          # also diverges from replay [1, 2]
    open(tl.STATE_PATH, "w").write(json.dumps(st))
    tampered_bytes = open(tl.STATE_PATH, "rb").read()
    r = tl.validate_state()
    c = r["checks"]["state_vs_events"]
    ok = (r["consistent"] is False and c["status"] == "fatal"
          and set(c["fields"]) == {"next_task", "completed"}
          and "next_task" in (c["detail"] or "")
          and open(tl.STATE_PATH, "rb").read() == tampered_bytes)
    print(f"{PASS if ok else FAIL} V2 drift fatal + report-only (state untouched)")
    if not ok:
        print(json.dumps(r, indent=2))
        return 1
    # Restore truth for later cases.
    open(tl.STATE_PATH, "wb").write(before_bytes)

    # ---- V3 — renumbered seq (replay-equivalent) -> only seq check fatal
    lines = open(tl.EVENTS_PATH, encoding="utf-8").read().splitlines()
    ev2 = json.loads(lines[1])
    ev2["seq"] = 9
    lines[1] = json.dumps(ev2, separators=(",", ":"))
    open(tl.EVENTS_PATH, "w").write("\n".join(lines) + "\n")
    st = read_state(tl)
    st["last_event_seq"] = 2      # align live count so ONLY seq fires
    open(tl.STATE_PATH, "w").write(json.dumps(st))
    r = tl.validate_state()
    ok = (r["consistent"] is False
          and r["checks"]["event_seq"]["status"] == "fatal"
          and "seq=9" in (r["checks"]["event_seq"]["detail"] or ""))
    print(f"{PASS if ok else FAIL} V3 seq gap fatal with offending detail")
    if not ok:
        print(json.dumps(r, indent=2))
        return 1
    # V4 rewrites both files wholesale; no restore needed here.

    # ---- V4 — replayed pointer landing on a currently-blocked id
    # Craft events by hand: complete 1, block 2, unblock 2, complete 2,
    # block 3 => replay pointer = 3 while 3 is blocked.
    events = [
        {"seq": 1, "ts": "2026-08-23T00:00:00Z", "event": "completed",
         "task_id": 1, "note": "", "evidence": []},
        {"seq": 2, "ts": "2026-08-23T00:00:01Z", "event": "completed",
         "task_id": 2, "note": "", "evidence": []},
        {"seq": 3, "ts": "2026-08-23T00:00:02Z", "event": "blocked",
         "task_id": 3, "note": "seed"},
    ]
    open(tl.EVENTS_PATH, "w").write(
        "".join(json.dumps(e, separators=(",", ":")) + "\n" for e in events))
    st = {
        "schema_version": 2, "ledger": "MASTER_TASK_LEDGER.jsonl",
        "total_tasks": 3, "next_task": 3, "completed": [1, 2],
        "blocked": [3], "skipped": [], "last_completed_at": None,
        "last_event_seq": 3,
    }
    open(tl.STATE_PATH, "w").write(json.dumps(st))
    r = tl.validate_state()
    ok = (r["consistent"] is False
          and r["checks"]["pointer_range"]["status"] == "fatal"
          and "currently blocked" in
          (r["checks"]["pointer_range"]["detail"] or ""))
    print(f"{PASS if ok else FAIL} V4 pointer-on-blocked fatal")
    if not ok:
        print(json.dumps(r, indent=2))
        return 1

    # ---- V6 — end-of-ledger None pointer stays consistent
    events.append({"seq": 4, "ts": "2026-08-23T00:00:03Z",
                   "event": "unblocked", "task_id": 3, "note": ""})
    events.append({"seq": 5, "ts": "2026-08-23T00:00:04Z",
                   "event": "completed", "task_id": 3, "note": "",
                   "evidence": ["docs/tasks/evidence/nonexistent-x.md"]})
    open(tl.EVENTS_PATH, "w").write(
        "".join(json.dumps(e, separators=(",", ":")) + "\n" for e in events))
    st.update({"next_task": None, "completed": [1, 2, 3], "blocked": [],
               "last_event_seq": 5})
    open(tl.STATE_PATH, "w").write(json.dumps(st))
    r = tl.validate_state()
    ok = (r["consistent"] is True            # warning NEVER flips consistency
          and r["checks"]["pointer_range"]["status"] == "ok"
          and r["checks"]["evidence"]["status"] == "warning"
          and len(r["checks"]["evidence"]["missing"]) == 1)
    print(f"{PASS if ok else FAIL} V6 end-of-ledger None consistent; "
          f"missing evidence warns without fatality")
    if not ok:
        print(json.dumps(r, indent=2))
        return 1

    # ---- V8 — orphan stub warning
    stub = tmp / "evidence" / "T-00099-completion.md"
    tmp.joinpath("evidence").mkdir(exist_ok=True)
    stub.write_text("# orphan\n")
    r = tl.validate_state()
    ok = (r["checks"]["evidence"]["status"] == "warning"
          and "T-00099-completion.md" in r["checks"]["evidence"]["orphans"]
          and r["checks"]["state_vs_events"]["status"] == "ok")
    print(f"{PASS if ok else FAIL} V8 orphan completion stub warned, "
          f"structural checks still ok")
    if not ok:
        print(json.dumps(r, indent=2))
        return 1

    # ---- V5/V7 — fresh ledger boundary + loud failures on bad inputs
    tmp2 = Path(tempfile.mkdtemp(prefix="task-validate-empty-"))
    tl2 = load_module(str(tmp2))
    make_ledger(tmp2 / "MASTER_TASK_LEDGER.jsonl", n=2)
    make_fresh_state(tmp2 / "TASK_STATE.json", total=2)
    r = tl2.validate_state()
    ok = r["consistent"] is True and r["replay"]["events"] == 0 \
        and r["replay"]["next_task"] == 1
    if ok:
        try:
            tl2.validate_state(state_path=str(tmp2 / "missing.json"))
            ok = False
        except FileNotFoundError:
            pass
    if ok:
        open(tmp2 / "COMPLETIONS.jsonl", "a").write("{corrupt\n")
        try:
            tl2.validate_state()
            ok = False
        except ValueError as e:
            ok = "corrupt event log" in str(e)
    print(f"{PASS if ok else FAIL} V5/V7 empty-log boundary; missing-state "
          f"and corrupt-event fail loudly, no partial findings")
    if not ok:
        return 1

    # ---- V9 — T-00108 hardening (F-1): hostile/absolute evidence paths
    # can NEVER satisfy the existence check, even when the target exists.
    events2 = [
        {"seq": 1, "ts": "2026-08-23T00:00:00Z", "event": "completed",
         "task_id": 1, "note": "", "evidence": [
             "/etc/passwd",                      # absolute, exists
             "../../../../etc/shadow",           # escapes both bases
             "docs/tasks/evidence",              # legit repo-relative dir? no file -> missing
         ]},
    ]
    open(tl.EVENTS_PATH, "w").write(
        "".join(json.dumps(e, separators=(",", ":")) + "\n" for e in events2))
    st = {"schema_version": 2, "ledger": "MASTER_TASK_LEDGER.jsonl",
          "total_tasks": 3, "next_task": 2, "completed": [1], "blocked": [],
          "skipped": [], "last_completed_at": None, "last_event_seq": 1}
    open(tl.STATE_PATH, "w").write(json.dumps(st))
    r = tl.validate_state()
    miss = " ".join(r["checks"]["evidence"]["missing"])
    ok = ("/etc/passwd" in miss and "../../.." in miss
          and r["checks"]["state_vs_events"]["status"] == "ok")
    print(f"{PASS if ok else FAIL} V9 hostile evidence paths always 'missing' "
          f"(existence oracle closed)")
    if not ok:
        print(json.dumps(r["checks"]["evidence"], indent=1))
        return 1

    print("PASS: validate unit tests (V1..V9)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
