# T-00120 — CI Smoke Orchestration / data model: Verification & Evidence

Date: 2026-08-23 · Status: COMPONENT CLOSED (10/10 lifecycle complete)

## Full baseline + epic suites — captured output

`AIOSH_CI_RESULTS=/tmp/opencode/t120-summary.json bash ci/run_all_smokes.sh`
→ exit 0, executed THROUGH the new integrated orchestrator:

```
PASS: rust_smoke
PASS: classifier_smoke
... (19/19)
PASS: task_ledger_scaffold (120 ms)
PASS: task_docs_unit (216 ms)
PASS: task_docs_scaffold (316 ms)
== ALL 19 SMOKE SUITES PASS (180515 ms) ==
```

Summary artifact verified: `total=19 passed=19 all_pass=true`, order
rust_smoke→task_docs_scaffold preserved. Rust unit suite: all 5 test
targets ok (82 tests). Epic suite: `tools/test_ci_suites.py` W1..W7 PASS
(mutation-proven at T-00115).

## Milestone bookkeeping

- `task_plan.md`: milestone entry "CI Smoke Orchestration data model
  CLOSED 10/10 (T-00111..T-00120)".
- `progress.md`: dated entry with shipped behavior + verification.

## Component summary

Research (G1..G5 gaps; TAP13 fetched-live prior art) → spec (registry +
record/summary schemas) → scaffold (complete registry data, loud-fail
functions) → implementation (validated constructor, atomic writer) →
unit tests (W1..W7 + mutation proof) → integration (`ci_run.py`
orchestrator; bash shim; full CI through new path) → security review
(S1..S7 incl. symlink-temp attack probe) → hardening (process-group kill,
bounded tails) → documentation (README section) → this closure.

Next component: CI Smoke Orchestration / core service at T-00121.
