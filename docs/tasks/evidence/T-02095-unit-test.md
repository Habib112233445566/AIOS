# Unit Test Evidence: T-02095 (recovery & validation: Unit Test)

- **Target File**: `code/aiosh-rust/aiosh-core/tests/test_capability_recovery.rs`
- **Subsystem**: Capability Model Recovery & Validation (`capability_recovery.rs`, `capability_service.rs`)
- **Execution Command**: `cargo test --test test_capability_recovery`
- **Test Results**:
  - `test_recovery_empty_service_validation`: **PASSED** (verifies empty service has zero caps, zero errors, healthy=true, invariants satisfied)
  - `test_recovery_valid_hierarchy_validation`: **PASSED** (verifies root and valid attenuated child pass structural checks, healthy=true)
  - `test_recovery_dangling_parent_validation`: **PASSED** (detects dangling parent reference, marks cap invalid, healthy=false)
  - `test_recovery_cycle_detection_validation`: **PASSED** (detects cyclic delegation lineage, marks both invalid, healthy=false)
  - `test_recovery_privilege_escalation_validation`: **PASSED** (detects child right not present in parent, healthy=false)
  - `test_recovery_missing_store_creates_default`: **PASSED** (verifies missing store file triggers `CreatedDefaultFresh` without errors)
  - `test_recovery_clean_existing_store`: **PASSED** (verifies valid store file loads cleanly via `LoadedExisting`)
  - `test_recovery_corrupted_store_quarantine`: **PASSED** (verifies corrupted store is non-destructively backed up to `.bak.<timestamp>` with mode 0600, returns `RecoveredFromBackup`, creates fresh store)
  - `test_recovery_report_invariants`: **PASSED** (verifies CAPREC1 `valid + invalid == total` and CAPREC2 `healthy == (errors.is_empty() && invalid == 0)`)
- **Status**: 9 passed; 0 failed; finished in 0.04s.
