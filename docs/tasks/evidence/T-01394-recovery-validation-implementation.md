# T-01394: Init & Service Supervision Recovery & Validation Implementation

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Recovery & Validation  
**Task ID:** T-01394  

---

## 1. Executive Summary
Task `T-01394` implemented the core operational behavior for the **Init & Service Supervision Recovery & Validation** subsystem in `code/aiosh-rust/aiosh-core/src/service_recovery.rs`. The module provides deep validation of managed service stores (enforcing naming, execution command traversal protection, timeout bounds, and topological acyclicity), timestamped quarantine backups of damaged files (`.corrupt.<ts>.bak`), and self-healing canonical reconstitution satisfying invariants `SR1..SR5`.

---

## 2. Implementation Details

### `code/aiosh-rust/aiosh-core/src/service_recovery.rs`
1. **`validate_service_store(store: &ServiceStore, store_path: &Path) -> ServiceValidationReport`:**
   - Evaluates total registered services against 10,000 capacity ceiling.
   - Asserts key-name identity (`key == spec.name`) and runs `validate_service_name` + `validate_service_spec`.
   - Asserts key-name identity on runtime statuses and runs `validate_service_status`.
   - Validates transitive dependency acyclicity by invoking `store.plan_service_order` across all services.
   - Calculates `valid_services`, `invalid_services`, and accumulated diagnostic errors.
   - Computes `healthy` flag and validates internal consistency invariants `SR1..SR3`.

2. **`create_backup_file(path: &Path) -> PathBuf`:**
   - Generates unique quarantine filename `<base>.corrupt.<epoch_ms>.bak` with collision counter.
   - Attempts atomic `fs::rename`; falls back safely to copy-and-remove across filesystems.

3. **`recover_service_store_with_backup(path: &Path) -> (ServiceStore, Option<PathBuf>)`:**
   - If path does not exist: initializes canonical `ServiceStore::new()` and persists to disk.
   - If store exists and passes validation: returns existing store without modification.
   - If store fails to load (malformed JSON) or fails validation: moves damaged file to timestamped backup and generates fresh canonical store.

4. **`load_or_recover(path: &Path) -> Result<(ServiceStore, ServiceValidationReport, bool, Option<PathBuf>), String>`:**
   - Unified orchestration loading existing store, validating health, and automatically repairing on demand.

---

## 3. Test Verification
The implementation is validated by comprehensive unit tests covering healthy validation, invalid spec detection, cyclic dependency detection, and corruption recovery:

```text
running 4 tests
test service_recovery::tests::test_validate_store_with_cyclic_dependency ... ok
test service_recovery::tests::test_validate_store_with_invalid_service ... ok
test service_recovery::tests::test_validate_default_store_healthy ... ok
test service_recovery::tests::test_recover_corrupt_store ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; finished in 0.03s
```
