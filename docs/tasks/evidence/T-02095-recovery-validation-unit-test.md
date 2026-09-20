# Unit Test Verification: T-02095

- **Task**: T-02095 (recovery & validation: Unit Test)
- **Suite**: `aiosh-core/tests/test_capability_recovery.rs`
- **Output**:
```
running 9 tests
test test_recovery_cycle_detection_validation ... ok
test test_recovery_dangling_parent_validation ... ok
test test_recovery_missing_store_creates_default ... ok
test test_recovery_empty_service_validation ... ok
test test_recovery_report_invariants ... ok
test test_recovery_clean_existing_store ... ok
test test_recovery_privilege_escalation_validation ... ok
test test_recovery_corrupted_store_quarantine ... ok
test test_recovery_valid_hierarchy_validation ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```
- **Invariants Verified**:
  - `CAPREC1`: `valid + invalid == total`
  - `CAPREC2`: `healthy == (errors.is_empty() && invalid == 0)`
  - `CAPREC3`: Lineage integrity and cycle detection
  - `CAPREC4`: Monotonic attenuation checks
  - `CAPREC5`: Non-destructive quarantine backup creation
  - `CAPREC6`: Safe persistence and atomic recovery
