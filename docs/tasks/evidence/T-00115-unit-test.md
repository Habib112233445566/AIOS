# T-00115 — CI Smoke Orchestration / data model: Unit Test

Date: 2026-08-23 · Status: UNIT TESTS COMPLETE

## New test file

`tools/test_ci_suites.py` (W-suite) — standalone, repo PASS/FAIL style,
never executes real CI suites (seconds-fast, hermetic tempdirs).

| Case | Asserts (observable behavior) |
|---|---|
| W1 | registry: 19 unique suites; `SUITE_NAMES` == regex-extracted `run_suite` order from `ci/run_all_smokes.sh`; every command's final arg exists on disk; timeouts positive |
| W2 | constructor happy path; `log_path` derived from shared template |
| W3 | `timeout`/`error` statuses force `exit_code=null` (even when caller passes an int — coercion is spec §3 mapping, pinned) |
| W4 | six invalid inputs rejected with field-naming errors (unknown suite / index-suite mismatch / bad status / negative duration / missing-Z timestamps / non-int index) |
| W5 | `write_summary` JSON round-trip; zero `*.tmp.*` leftovers |
| W6 | failed write (ENOTDIR parent — uid-independent; chmod checks are meaningless under root) leaves NO temp files |
| W7 | corrupted registry copy (duplicated suite name) fails AT IMPORT with "duplicate suite name" |

## Mutation-sensitivity proof (task requirement)

Neutered the status check (`if False and status not in …`) in
`tools/ci_suites.py`: suite FAILED at W4 ("accepted invalid input");
restored original: green. The suite demonstrably fails when the feature
breaks.

## Fixture bug caught during authoring (honest log)

W6 originally used `chmod 0o500` + `mkdir` to force a write failure —
meaningless in this container because the process runs as **root**, which
ignores DAC bits (the write SUCCEEDED and the suite caught it). Rewritten
to a uid-independent ENOTDIR parent (file where a directory is required).

## Verification

- `python3 tools/test_ci_suites.py` → PASS W1..W7, exit 0.
- Mutation run fails at W4; restored run passes.
- No regressions: U-suite green; no other module imports ci_suites yet.
