# T-00125 — CI Smoke Orchestration / core service: Unit Test

Date: 2026-08-23 · Status: UNIT TESTS COMPLETE

## New test file

`tools/test_ci_service.py` (X-suite) — standalone, repo PASS/FAIL style,
synthetic artifacts in hermetic tempdirs, never runs CI.

| Case | Asserts |
|---|---|
| X1 | valid artifact: `show` emits the exact spec §5 header/counters/row lines; `check` → exit 0, `ci-check: PASS (19/19 suites)` |
| X2 | seeded failure: `check` exit 1 with correct counts; `failures` lists the row (index, suite, duration, exit code, log path) |
| X3 | ELEVEN strict-load rejections, each exit 2 naming the field: missing key, schema_version=2, arithmetic incoherence, wrong all_pass, bad status, non-Z timestamp, reversed order, null exit on fail row, unknown suite, corrupt JSON, missing file |
| X4 | usage errors exit 2: no action, unknown token, double action, --file without value |
| X5 | boundary: incomplete clean run (`total=5, failed=0`) loads fine but FAILS the gate (`all_pass` false by construction) — fail-fast semantics preserved end-to-end |
| X6 | zero failures ⇒ `no failed suites` line |
| X7 | mutation sensitivity: neutering the schema-version check makes the mutant ACCEPT a v2 artifact (proving the suite bites); restored copy green |

## Fixture bug caught during authoring (honest log)

Missing-file assertion expected stderr to contain "not found"; Python's
FileNotFoundError text is "No such file …". Fixed assertion.

## Verification

- `python3 tools/test_ci_service.py` → PASS X1..X7, exit 0.
- No regressions: service module is read-only; U/W/V suites unaffected.
