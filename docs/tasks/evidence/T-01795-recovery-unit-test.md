# Task Evidence: T-01795 - Hardware Detection / Recovery & Validation: Unit Test

## Metadata
- **Task ID:** `T-01795`
- **Sub-Epic:** Sub-Epic 10: Hardware Detection / Recovery & Validation
- **Component:** `aiosh-core::tests::test_hardware_recovery`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Unit Test Suite Structure
1. `test_hval1_hval2_hval3_healthy_inventory_validation`:
   - Validates that a structurally sound inventory satisfies invariants `HVAL1..HVAL3`, reports `healthy == true`, and contains no summary mismatches.
2. `test_hval1_hval2_corrupted_inventory_in_memory_recovery`:
   - Tests surgical pruning of devices with invalid names, duplicate IDs, and malformed hex IDs.
   - Verifies automatic summary recalculation (`HVAL2`) and post-recovery health restoration.
3. `test_hval4_unparseable_json_quarantine_and_recovery`:
   - Validates that unparseable or corrupted JSON files on disk are safely backed up to `.bak.<timestamp>` (`HVAL4`) before initializing a clean store.
4. `test_hval5_oversized_store_file_rejection`:
   - Verifies that store files exceeding `MAX_STORE_FILE_SIZE` (10 MB) are safely rejected.
5. `test_hval6_sysfs_drift_detection`:
   - Verifies that inventory records referencing non-existent sysfs paths are flagged as drift when `check_paths` is enabled.
6. `test_hardware_service_validate_and_recover_store`:
   - Tests `HardwareService::validate_store` and `HardwareService::recover_store` methods.
