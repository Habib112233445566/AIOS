# T-00113 — CI Smoke Orchestration / data model: Scaffold

Date: 2026-08-23 · Status: SCAFFOLD COMPLETE

## What was added

New module `tools/ci_suites.py` (stdlib only):

| Element | Kind | State at scaffold |
|---|---|---|
| `SuiteDef`, `ResultRecord`, `RunSummary` TypedDicts | types | complete |
| `SUITES` registry (19 entries) + `SUITE_NAMES` | static data | COMPLETE — names/order/commands mirror `ci/run_all_smokes.sh` 1:1; per-suite timeouts (`rust_smoke` 1800 s, others 900 s) |
| Import-time validation (spec §5) | behavior | complete — duplicate name / empty command / bad timeout raise `ValueError` at load |
| `LOG_TEMPLATE`, `RESULTS_PATH` (`AIOSH_CI_RESULTS` override) | constants | complete |
| `build_result_record(...)` | function | loud-fail stub ("scaffolded (T-00113); lands in T-00114") |
| `write_summary(...)` | function | loud-fail stub |
| `summary_to_json(...)` | function | implemented (pure serialization used by stubs' contract tests later) |

## Call-site / test-stub referencing (acceptance #2)

- Registry referenced by the verification cross-check below AND by
  `SUITE_NAMES` consumers to come; functions referenced by this probe:
  `build_result_record()` raises as specified.

## Build/import verification

- `import ci_suites` → ok; 19 suites; loud NotImplementedError/TypeError
  from the stub constructor.
- **1:1 order-match against bash proven**: regex-extracted `run_suite`
  invocations from `ci/run_all_smokes.sh` == `SUITE_NAMES`
  (19 == 19, tuple equality True).
- No existing file touched; nothing else imports it yet → zero regression
  surface (U-suite re-run below still green).
- `python3 tools/test_task_ledger.py` → PASS U1..U16 (unchanged).

## Notes

- TypedDicts chosen over dataclasses for JSON-shape fidelity with the
  repo's serde_json/TypedDict conventions elsewhere.
