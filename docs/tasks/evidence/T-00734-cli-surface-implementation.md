# T-00734 — Secrets & Access Hygiene / CLI surface: Implementation

## 1. Implementation Summary
- Built complete CLI subcommand handler `cmd_secrets` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `aiosh secrets scan`: Performs full workspace or single-file scan and renders human-readable table or JSON envelope with finding cards and summary metadata.
  - `aiosh secrets check`: Fast status check outputting concise `[+]` pass or `[-]` fail messages suitable for CI checks.
  - Flag parsing: `--repo <path>`, `--file <path>`, `--max-bytes <n>`, `--json`.
  - Audit logging: Calls `emit()` generating SQLite WAL audit row (`secrets.scan` or `secrets.check`).
  - Standard exit codes: 0 (clean), 1 (secrets detected or error), 2 (usage syntax error).

## 2. Verification
- Targeted unit tests in `task_cli_tests::test_cmd_secrets_scan_and_check` passing.
- Full cargo build and test suite verified passing.
