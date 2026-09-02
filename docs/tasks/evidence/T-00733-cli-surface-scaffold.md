# T-00733 — Secrets & Access Hygiene / CLI surface: Scaffold

## 1. Scaffold Deliverables
- Registered `"secrets"` subcommand dispatch in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `Some("secrets") => cmd_secrets(&args[1..]),`
  - Integrated `aiosh secrets <scan|check>` into CLI `--help` / usage text.
- Implemented `cmd_secrets(args: &[String]) -> i32` skeleton supporting `--repo`, `--file`, `--max-bytes`, and `--json`.
- Added test harness `test_cmd_secrets_scan_and_check` in `aiosh-cli::task_cli_tests`.
- Verified compilation and test pass via `cargo test --bin aiosh`.
