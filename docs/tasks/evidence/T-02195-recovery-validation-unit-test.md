# Task Evidence: T-02195 (recovery & validation: Unit Test)

## 1. Scope & Execution
Authored focused, comprehensive automated tests for PEP Decision Engine Recovery & Validation subsystem in `code/aiosh-rust/aiosh-core/tests/test_pep_recovery.rs`.

## 2. Test Coverage Matrix
- `test_pep_recovery_valid_store_array_and_object`: Validates both JSON Array and JSON Object map representations (`rules: [...]` and `rules: { "r1": {...} }`).
- `test_pep_recovery_duplicate_id_detection`: Verifies duplicate rule ID rejection (`PEPRECV_ERR_DUPLICATE_ID`).
- `test_pep_recovery_semantic_target_validation`:
  - Path traversal in `target_resource` (`..`).
  - Control characters in `target_subject`.
  - Length overflow in `target_action` (> 64 chars).
- `test_pep_recovery_capacity_bounds`: Tests rule count exceeding 5,000 generating `PEPRECV_ERR_CAPACITY`.
- `test_pep_recovery_file_path_hygiene`: Tests non-existent files and path traversal (`..`).
- `test_pep_recovery_salvage_and_quarantine_strategy`: Verifies partial recovery on a 4-rule mixed store (2 valid, 2 invalid); confirms 2 rules salvaged, 2 dropped, backup `.bak` file created, and new store sanitized.
- `test_pep_recovery_strict_fail_closed`: Verifies strict fail-closed resets store to 0 rules and moves damaged file to quarantine.
- `test_pep_recovery_dry_run`: Verifies dry run reports validation issues without altering disk contents.

## 3. Test Execution Results
```
running 8 tests
test test_pep_recovery_duplicate_id_detection ... ok
test test_pep_recovery_file_path_hygiene ... ok
test test_pep_recovery_semantic_target_validation ... ok
test test_pep_recovery_dry_run ... ok
test test_pep_recovery_strict_fail_closed ... ok
test test_pep_recovery_salvage_and_quarantine_strategy ... ok
test test_pep_recovery_valid_store_array_and_object ... ok
test test_pep_recovery_capacity_bounds ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
```

## 4. Acceptance Confirmation
- [x] Dedicated test suite runs standalone and passes cleanly (8/8 tests).
- [x] Negative, boundary, and corruption cases verified.
- [x] Observable disk state, backup files, and return structures asserted.
