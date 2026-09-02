# T-00854 — Regression Triage / Configuration: Implementation

## 1. Implementation Deliverables
- Implemented `TriageStore::ingest_ci_summary_with_config` and `TriageStore::load_from_path_with_config` in `code/aiosh-rust/aiosh-core/src/triage_service.rs`.
- Added suite filter matching `should_ingest_suite` in `code/aiosh-rust/aiosh-core/src/triage_config.rs`.
- Added criterion `T5` to `tools/test_triage_suites.py`.
- Verified execution through `triage_service::tests::test_store_config_integration` and `triage_config::tests::test_triage_config_should_ingest_suite`.
