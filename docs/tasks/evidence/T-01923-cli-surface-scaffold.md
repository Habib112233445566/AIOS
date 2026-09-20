# Task Evidence: T-01923 - System Update / CLI surface: Scaffold

- **Task**: `T-01923`
- **Sub-Epic**: `Sub-Epic 3: Operator CLI & Control Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Scaffolded the CLI surface entrypoint and subcommands for the System Update subsystem in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Routed `Some("update") | Some("upd")` to `cmd_update(&args[1..])`.
- Implemented subcommand parsing for `status`, `slots`, `check`, `apply`, `confirm`, and `rollback`.
- Handled global flags `--state-dir <path>`, `--staging-dir <path>`, and `--json`.
- Enforced input path hygiene checks for path length limit (<= 1024) and control character rejection.
- Provided structured JSON output envelopes (`code`, `data`, `error`) and human-readable terminal rendering.
- Integrated structured audit logging using `classify_and_emit`.

## Verification
- Verified compilation via `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli`.
