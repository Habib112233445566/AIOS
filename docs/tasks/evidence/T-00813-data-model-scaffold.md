# T-00813 — Regression Triage / data model: Scaffold

## 1. Module Skeleton & Exports
- Created `code/aiosh-rust/aiosh-core/src/triage.rs` defining:
  - `TriageStatus`: Lifecycle states (`Untriaged`, `Triaged`, `FixPending`, `Resolved`, `WontFix`).
  - `TriageSeverity`: Priorities (`Blocker`, `Critical`, `Major`, `Minor`).
  - `TriageRecord`: Granular failure tracking struct with SHA-256 deduplication signatures and occurrences counter.
  - `TriageReport`: Aggregated report struct with validation helper `validate_triage_report`.
  - `compute_failure_signature`: Normalized SHA-256 failure signature calculator using `crate::canonical::sha256_hex`.
- Exported `pub mod triage;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Unit tests in `triage::tests` (4/4 PASS).
