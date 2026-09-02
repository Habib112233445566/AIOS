# T-00833 — Regression Triage / CLI: Scaffold

## 1. CLI Skeleton & Registration
- Scaffolded `cmd_triage` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Registered `triage` subcommand with subcommands: `list`, `show`, `record`, `resolve`, `ingest`, `check`.
- Verified compilation and CLI tests in `test_cmd_triage_flow`.
