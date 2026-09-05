# T-01295: Package Management Recovery & Validation Unit Test

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Recovery & Validation  
**Task ID:** T-01295  

---

## 1. Unit Test Overview
Task `T-01295` adds a focused automated test suite in `code/aiosh-rust/aiosh-core/tests/test_package_recovery.rs` verifying recovery and validation invariants `RV1..RV4`.

---

## 2. Test Cases Implemented
1. **`test_default_store_validation`**:
   - Asserts default reference store contains $\ge 5$ packages, is healthy, with 0 invalid packages and empty error list.
   - Verifies all invariants `RV1..RV3` hold.
2. **`test_invariant_equations_rv1_rv2_rv3`**:
   - Explicitly asserts failure modes:
     - `RV1` violation (count divergence between valid + invalid and total).
     - `RV2` violation (healthy set to true despite errors or invalid packages).
     - `RV3` violation (number of errors less than invalid packages).
   - Confirms valid report passes validation.
3. **`test_negative_package_specs_and_store_constraints`**:
   - Tests uppercase names, empty versions, self-referential dependencies, invalid SHA-256 strings, and mismatched map keys.
   - Validates that every invalid package is identified and counted.
4. **`test_recover_corrupted_json_store_rv4`**:
   - Simulates broken/incomplete JSON buffer on disk.
   - Verifies `load_or_recover` triggers recovery.
   - Verifies timestamped backup exists and preserves exact damaged bytes (`RV4`).
   - Verifies newly reseeded store is healthy and persisted.
5. **`test_recover_missing_store_file`**:
   - Tests zero-downtime bootstrapping when store file does not exist.
6. **`test_healthy_store_no_recovery`**:
   - Ensures clean stores are not unnecessarily backed up or reseeded.
7. **`test_corrupted_store_invalid_specs_triggers_backup`**:
   - Ensures structurally malformed packages written to disk are quarantined.
8. **`test_recover_package_store_with_backup_direct`**:
   - Directly tests missing vs corrupted store recovery helpers.
