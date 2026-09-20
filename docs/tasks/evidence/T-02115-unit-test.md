# Unit Test Evidence: T-02115 (PEP Decision Engine / core service: Unit Test)

- **Target File**: `code/aiosh-rust/aiosh-core/tests/test_pep_decision_service.rs`
- **Subsystem**: PEP Decision Engine Core Service (`pep_decision_service.rs`)
- **Execution Command**: `cargo test --test test_pep_decision_service`
- **Test Scenarios**:
  - `test_service_new_empty`: **PASSED** (verifies empty service has len 0, is_empty true, default algorithm DenyOverrides)
  - `test_service_add_and_get_rule`: **PASSED** (verifies rule addition, indexing, and ID lookup)
  - `test_service_remove_rule`: **PASSED** (verifies rule removal and index cleanup)
  - `test_service_evaluation`: **PASSED** (verifies request evaluation via service)
  - `test_service_capacity_limit`: **PASSED** (verifies `MAX_RULES_IN_SERVICE` 5,000 capacity ceiling)
  - `test_service_save_and_load`: **PASSED** (verifies atomic persistence and loading from JSON)
  - `test_service_load_or_recover_corrupt`: **PASSED** (verifies non-destructive quarantine to `.bak.<timestamp>` with mode 0600 on corrupted files)
  - `test_service_path_traversal_rejected`: **PASSED** (verifies rejection of `..` traversal and non-`.json` paths)
- **Result**: 8 passed; 0 failed; finished in 0.36s. Zero warnings.
