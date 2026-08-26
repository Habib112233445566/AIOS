# T-00114 — CI Smoke Orchestration / data model: Implementation

Date: 2026-08-23 · Status: IMPLEMENTATION COMPLETE

> Bookkeeping note (honest log): the completion event for this task
> (seq 114 in `COMPLETIONS.jsonl`) cites
> `docs/tasks/evidence/T-00113-scaffold.md` instead of this file — a
> copy-paste slip in the `--evidence` argument. The event log is
> append-only by design, so the wrong reference stands in history; THIS
> file is the correct implementation evidence for T-00114.

## What shipped (`tools/ci_suites.py`, replacing scaffold bodies)

1. `build_result_record(...)` — validated pure constructor:
   - suite must exist in the registry AND `index` must be its registry
     position (order-is-contract enforced in data, spec §3);
   - `status` restricted to `pass|fail|timeout|error`;
   - `timeout`/`error` records force `exit_code = null` (spec mapping);
     `pass`/`fail` demand an int;
   - `duration_ms` non-negative int; timestamps must end in `Z`;
   - `log_path` derived from the shared `LOG_TEMPLATE`.
   Every rejection names the offending field (loud-error rule).
2. `write_summary(summary, path=None)` — atomic artifact writer:
   `O_EXCL` temp → write → fsync → `os.replace`; temp unlinked on any
   error (no orphan temps); returns the written path; defaults to
   `AIOSH_CI_RESULTS`/`/tmp/aiosh-ci-results.json`.

No new dependencies; registry/types untouched from scaffold.

## Verification (commands + results)

- Happy path: pass-record with derived log path; timeout-record with
  forced null exit — both construct cleanly.
- Rejections: unknown suite / index-suite mismatch / bad status /
  negative duration → `ValueError` with expected fragment (4/4).
- Round-trip: `write_summary` → `json.load` equals
  `json.loads(summary_to_json(s))`; zero `*.tmp.*` leftovers.
- Regression: `python3 tools/test_task_ledger.py` → PASS U1..U16.
