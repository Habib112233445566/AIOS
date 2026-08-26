# T-00105 — Task Ledger Control / recovery & validation: Unit Test

Date: 2026-08-23 · Status: UNIT TESTS COMPLETE

## New test file

`tools/test_task_validate.py` (V-suite) — standalone, same harness style as
`tools/test_task_ledger.py` (temp-dir sandbox via `AIOSH_TASKS_DIR`,
PASS/FAIL markers, non-zero exit on failure, zero contact with real
docs/tasks state).

| Case | Asserts (observable behavior only) |
|---|---|
| V1 | clean sandbox → `consistent:true`; exact checks key set `{state_vs_events,event_seq,pointer_range,evidence}`; replay==live pointers |
| V2 | **primary failure mode** drift: tampered `next_task`+`completed` → `fatal`, `fields` names both, state file byte-identical after run (report-only proof) |
| V3 | renumbered event seq (replay-equivalent munge) → seq check `fatal` with offending detail; other checks unaffected |
| V4 | replayed pointer landing on a currently-blocked id → pointer_range `fatal` |
| V5 | boundary: fresh ledger + empty event log → consistent |
| V6 | boundary: end-of-ledger `null` pointer stays consistent; a missing evidence path yields `warning` WITHOUT flipping consistency |
| V7 | invalid input: missing state file → loud `FileNotFoundError`; corrupt event line → `ValueError("corrupt event log…")`; never partial findings |
| V8 | orphan `T-00099-completion.md` stub → evidence warning; structural checks still ok |

## Mutation-sensitivity proof (task requirement)

Neutered the drift comparison (`drift_ok = not drift_fields` → `True`) in
`tools/task_ledger.py`: suite **FAILED at V2** (exit 1). Restored the
original: suite green again (exit 0). The suite demonstrably fails when the
feature is broken.

## Fixture bugs caught during authoring (honest log)

1. V2 initially tampered `next_task` to the replay-equal value → only one
   field diverged; fixed fixture to diverge both fields.
2. Report-only assertion originally compared post-run bytes to PRE-tamper
   bytes instead of the tampered bytes; fixed.
3. V6/V8 fixture completed a still-blocked task with no unblock event —
   the validator correctly flagged the resulting drift (`blocked live=[]
   replay=[3]`), proving replay semantics are enforced; fixture rewritten
   to the legal block→unblock→complete sequence.
4. V6 asserted warnings flip `consistent` — contradicts spec §4
   (warning-only); assertion corrected to match the contract.

## Verification

- `python3 tools/test_task_validate.py` → **PASS V1..V8**, exit 0.
- Mutation run fails at V2 as designed; restored run passes.
- No regressions: `python3 tools/test_task_ledger.py` PASS U1..U16;
  Rust side untouched by this task but full `cargo test` re-run → all 5
  targets ok (82 tests).
