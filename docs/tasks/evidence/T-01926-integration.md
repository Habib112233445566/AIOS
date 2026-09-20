# Task Evidence: T-01926 - System Update / CLI surface: Integration

- **Task**: `T-01926`
- **Sub-Epic**: `Sub-Epic 3: Operator CLI & Control Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Authored and verified end-to-end CLI integration smoke test suite `code/aiosh-cli/tests/test_system_update_cli_smoke.py` invoking the compiled Rust binary `aiosh.exe`:
- Tested alias invocation: `aiosh update` and `aiosh upd`.
- Tested help banner and unknown subcommands (`UNKNOWN_SUBCOMMAND`, exit code 2).
- Tested security path hygiene on `--state-dir` and `--staging-dir` against length limits (>1024) and control character injection (`PATH_TOO_LONG`, `PATH_CONTAINS_CONTROL_CHAR`, exit code 2).
- Tested `status` and `slots` subcommands in both human-readable and structured JSON envelope modes (`code: 0`).
- Tested `check` subcommand against missing argument (`MISSING_MANIFEST_PATH`, exit code 2), missing file (`READ_ERROR`, exit code 1), invalid JSON (`INVALID_JSON`, exit code 1), and valid manifest payload (`code: 0`, transition to `Downloading`).
- Tested `confirm` and `rollback` subcommands under error states (Idle state or missing rollback slot returning exit code 1) and valid `ReadyToReboot` state returning exit code 0 and verified persistence.

## Verification
- Executed `python code/aiosh-cli/tests/test_system_update_cli_smoke.py`.
- Output:
  ```
  Running System Update CLI smoke tests against C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\aiosh.exe...
  PASS: test_update_help_and_unknown
  PASS: test_update_path_hygiene
  PASS: test_update_status_and_slots
  PASS: test_update_check_and_validation
  PASS: test_update_confirm_and_rollback
  All System Update CLI smoke tests PASSED successfully.
  ```
