# T-00096 — documentation component: Integration (evidence)

**Date:** 2026-08-22
**Result:** checker + both suites reachable through the production CI
surface; full baseline green end-to-end.

## Registration point (discoverability)

`ci/run_all_smokes.sh` now runs, after the ledger suites:

```
run_suite task_docs_unit     tools/test_task_docs.py        (U1–U18)
run_suite task_docs_scaffold tools/test_task_docs_scaffold.py
```

The checker itself is invoked by the unit suite and directly runnable:
`python3 tools/check_task_docs.py` (documented operator invocation
lands with T-00099).

## Cross-substrate parity

N/A **by verified construction**: the checker touches no SQLite DB and
no canonical-JSON hashing (read-only text invariants). Confirmed by
inspecting its I/O surface: only `Path.read_text` on five doc files.

## Integration smoke — FULL baseline (captured live)

```
bash ci/run_all_smokes.sh → ALL 19 SMOKE SUITES PASS
```

(17 prior suites + the two newly wired doc suites; rust_smoke,
metrics_smoke, matrix M1–M8 all still green.)

## Acceptance

- [x] Feature reachable through its production surface (CI runner;
      standalone CLI invocation also works)
- [x] Integration smoke passes end-to-end (19/19)
