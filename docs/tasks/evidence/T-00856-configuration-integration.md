# T-00856 — Regression Triage / Configuration: Integration

## 1. Integration Deliverables
- Integrated `--config <path>` flag into `aiosh triage` CLI subcommands in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Supported fallback resolution hierarchy (`--store` flag -> `config.store_path` -> `$AI_HOME/triage_store.json`).
- Updated `aiosh triage ingest` to apply configured suite filters and default severity.
- Verified CLI integration flow via `task_cli_tests::test_cmd_triage_flow`.
