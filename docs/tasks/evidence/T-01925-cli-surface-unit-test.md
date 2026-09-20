# Task Evidence: T-01925 - System Update / CLI surface: Unit Test

- **Task**: `T-01925`
- **Sub-Epic**: `Sub-Epic 3: Operator CLI & Control Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Authored and executed unit tests for the System Update CLI operator interface in `code/aiosh-rust/aiosh-cli/src/main.rs`:
- `test_update_cli_help_and_subcommands`: Validates `--help`, `-h`, empty args (exit code 0), and unknown subcommands (exit code 2).
- `test_update_cli_path_hygiene`: Validates rejection of oversized paths (>1024 bytes) and control characters (`\n`, `\t`) for `--state-dir` and `--staging-dir` with exit code 2.
- `test_update_cli_status_and_slots`: Validates `status` and `slots` subcommands in both human-readable and `--json` format with exit code 0.
- `test_update_cli_confirm_and_rollback`: Validates `confirm` and `rollback` operations under error conditions (unsupported state or missing fallback) and success conditions (ReadyToReboot state).
- `test_update_cli_check`: Validates error handling on non-existent manifest files and malformed JSON files returning exit code 1.

## Verification
- Executed `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli update_cli_tests`.
- Output: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.94s`.
