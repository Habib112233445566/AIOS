# T-00853 — Regression Triage / Configuration: Scaffold

## 1. Scaffold Deliverables
- Scaffolded `code/aiosh-rust/aiosh-core/src/triage_config.rs` and registered in `lib.rs`.
- Declared `TriageConfig` with `max_store_bytes`, `default_severity`, `auto_ingest_suites`, `retention_days`, `notify_blockers`, and `store_path`.
- Implemented `validate()`, `from_file()`, `from_env_or_default()`, and `save_to_file()`.
- Verified compilation and unit tests in `triage_config::tests`.
