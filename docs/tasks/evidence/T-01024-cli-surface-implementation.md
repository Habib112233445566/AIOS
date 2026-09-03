# T-01024 — Distro Selection & Justification / CLI Surface: Implementation

## 1. Implementation Summary
- Extended `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - Connected `cmd_distro` to `main()` argument parser.
  - Implemented `list`: Formatted tabular output and JSON serialization.
  - Implemented `show`: Validated target ID retrieval with full specification reporting.
  - Implemented `evaluate`: Calculated multi-factor scores for single or all profiles.
  - Implemented `recommend`: Displayed default reference Linux profile for AIOS.
  - Added audit logging calls via `classify_and_emit` to guarantee immutable record emission for each action.
- Executed `test_cmd_distro_flow` inside `aiosh-cli` and `test_distro_cli_smoke.py`.

## 2. Test Execution Output
```
running 1 test
test task_cli_tests::test_cmd_distro_flow ... ok

PASS: aiosh distro list prose
PASS: aiosh distro list --json
PASS: aiosh distro show prose
PASS: aiosh distro show --json
PASS: aiosh distro evaluate --json
PASS: aiosh distro evaluate <id> --json
PASS: aiosh distro recommend --json
PASS: aiosh distro --help
PASS: aiosh distro show missing id returns 2
PASS: aiosh distro show nonexistent returns 1

ALL DISTRO CLI SMOKE TESTS PASSED!
```
