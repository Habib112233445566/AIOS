# T-01195: Base Image Build Recovery & Validation Unit Test

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Recovery & Validation  
**Task ID:** T-01195  

## 1. Test Suite Implementation
Implemented focused automated unit test suite in `code/aiosh-rust/aiosh-core/tests/test_base_image_recovery.rs` covering:
1. `test_default_store_validation`: Asserts default seeded store is 100% healthy, has 0 invalid manifests, 0 errors, and passes `validate_invariants()`.
2. `test_manifest_boundary_and_negative_rules`: Asserts rejection of empty IDs, oversized IDs (>128 chars), control characters in IDs, unauthorized architectures, unauthorized filesystems, empty packages, blacklisted packages (`rsh-client`), control characters in package names, dangerous kernel parameters (`mitigations=off`), and zero size budgets.
3. `test_invariant_violations_rv1_rv2_rv3`: Validates error detection for forged reports violating invariants `RV1`, `RV2`, or `RV3`.
4. `test_corruption_recovery_and_backup_creation`: Validates the full recovery lifecycle:
   - Missing path: creates default fresh store.
   - Valid path: loads existing store.
   - Malformed JSON: creates timestamped `.bak` file and reseeds clean store.
   - Schema invalid JSON: detects violation, creates timestamped `.bak` file, and reseeds clean store.
5. `test_repair_store_api`: Validates explicit store repair.

## 2. Test Execution Output
```
running 5 tests
test test_invariant_violations_rv1_rv2_rv3 ... ok
test test_default_store_validation ... ok
test test_manifest_boundary_and_negative_rules ... ok
test test_repair_store_api ... ok
test test_corruption_recovery_and_backup_creation ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```
Status: PASS (Exit code 0).
