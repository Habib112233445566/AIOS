# Task Evidence: T-01873 - Network Bootstrap / observability: Scaffold

## 1. Overview
- **Task ID**: `T-01873`
- **Sub-Epic**: 8 (Network Bootstrap Observability)
- **Goal**: Scaffold module skeleton and public interfaces for Network Bootstrap Observability in `code/aiosh-rust/aiosh-core/src/network_observability.rs`.

---

## 2. Scaffold Deliverables
- Created `code/aiosh-rust/aiosh-core/src/network_observability.rs` defining:
  - Error constants: `NOBS_IO_ERROR`, `NOBS_PARSE_ERROR`, `NOBS_PATH_ERROR`, `NOBS_VALIDATION_ERROR`.
  - Constants: `MAX_OBSERVABILITY_FILE_BYTES` (1 MB), `DEFAULT_HISTORY_CAPACITY` (60).
  - Data structures: `InterfaceStatistics`, `NetworkHealthVerdict`, `NetworkHealthReport`, `NetworkObservabilitySnapshot`.
  - Service: `NetworkObservabilityService` with `collect_statistics`, `evaluate_health`, `capture_snapshot`, `get_history`, `save_snapshot_to_path`, `load_snapshot_from_path`.
  - Path hygiene validator: `validate_observability_path()`.
- Re-exported `pub mod network_observability;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Validated via `cargo check` (exit code 0, 0 compiler errors).
