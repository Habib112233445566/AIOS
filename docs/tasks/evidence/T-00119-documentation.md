# T-00119 — CI Smoke Orchestration / data model: Documentation

Date: 2026-08-23 · Status: DOCUMENTATION COMPLETE

## What shipped (operator view)

New **"CI Smoke Orchestration (T-00111..T-00120)"** section in
`docs/README.md`:

- Delegation model: `ci/run_all_smokes.sh` → `tools/ci_run.py` → registry
  `tools/ci_suites.py` (single source; order-is-contract warning).
- Copy-pasteable examples: full CI run with custom `AIOSH_CI_RESULTS`
  location + a one-liner programmatic consumer of the summary artifact.
- Summary schema documented inline (stable additive-only key set; status
  enum).
- Limitations stated, not omitted: double-fork can escape group-kill;
  on-disk logs uncapped (memory bounded to 64 KiB tail); summary is
  advisory telemetry — exit code remains the CI verdict.
- Evidence range linked (`tasks/evidence/T-00111…T-00120`).

## Structural note (honest log)

The first edit spliced the new section between the existing
"Documentation invariants" heading and its body, briefly duplicating the
heading. Caught by re-reading the rendered region; deduplicated to
sibling `##` sections.

## Invariant compliance

`tools/check_task_docs.py` → C1..C6 green after edits; docs test suite
green. All backticked paths resolve (C3), no TODO markers, no volatile
CI counts.
