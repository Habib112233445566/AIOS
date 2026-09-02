# T-00823 — Regression Triage / core service: Scaffold

## 1. Module Skeleton & Exports
- Created `code/aiosh-rust/aiosh-core/src/triage_service.rs` defining:
  - `TriageStore`: Store structure with `HashMap` indices for signatures and `TRG-xxxxxxxx` IDs.
  - `record_failure`: Deduplication and recurrence counter logic.
  - `ingest_ci_summary`: Automatic regression ingestion from `ci::RunSummary`.
  - `resolve` and `update_status`: Lifecycle status mutation.
  - `save_to_path` and `load_from_path`: File persistence with 1 MiB size cap.
- Exported `pub mod triage_service;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Unit tests in `triage_service::tests` (3/3 PASS).
