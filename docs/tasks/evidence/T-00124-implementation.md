# T-00124 — CI Smoke Orchestration / core service: Implementation

Date: 2026-08-23 · Status: IMPLEMENTATION COMPLETE

## What shipped (`tools/ci_service.py`, replacing scaffold bodies)

1. `load_summary(path)` — the strict validator-loader (spec §4):
   JSON-parse guard; required-key set (additive tolerated);
   `schema_version == 1` with explicit refusal to best-effort parse;
   non-negative-int counters; `passed+failed == total` coherence;
   `all_pass` recomputed-and-compared against registry size; per-row
   registry membership + index-position + strictly-increasing order +
   status enum + null-exit-code-only-for-timeout/error + Z-timestamps +
   non-negative duration. Every violation names the field.
2. `failure_rows` — status!=pass projection in run order.
3. `human_report` — exact spec §5 line formats (header, counters,
   `[ok ]`/`[FAIL]` rows with duration/exit/log).
4. `check` action — gate semantics: exit 0 iff `all_pass` AND complete
   run; one-line verdict `ci-check: PASS|FAIL (n/19 suites, k failed)`.
   Counts come from the artifact (report-only; never recomputed from a
   live registry state).
5. Cleanup: removed an unused `argparse` import left from scaffold.

## Verification (live)

- Real artifact from the T-120 full run: `show` renders the report;
  `check` → `ci-check: PASS (19/19 suites)`, exit 0.
- Seeded-fail artifact (pentest_smoke forced to fail, counters adjusted):
  `check` → exit 1 `ci-check: FAIL (18/19 suites, 1 failed)`;
  `failures` lists exactly that row with exit code + log path.
- Loud-error paths verified at scaffold (usage/exit 2) and remain.

## Acceptance mapping

- Targeted behavior passes on both real and seeded artifacts ✅.
- No regression surface: read-only new module; no existing file's
  behavior touched (U/W suites unaffected — re-run in T-00125).
