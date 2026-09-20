# Task Evidence: T-01798 - Hardware Detection / Recovery & Validation: Hardening

## Metadata
- **Task ID:** `T-01798`
- **Sub-Epic:** Sub-Epic 10: Hardware Detection / Recovery & Validation
- **Component:** `aiosh-core::hardware_recovery`, `aiosh-core::tests::test_hardware_recovery`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Hardening Remediations Implemented
1. **Path Traversal & Control Character Validation (`validate_store_path`)**:
   - Remediation for `THREAT-HREC-01`: Added `validate_store_path` which inspects path components for parent directory traversal (`..`), checks for control characters (`c.is_control()`), enforces `MAX_PATH_LEN = 1024`, and mandates the `.json` extension.
   - Enforced across both `check_inventory_file` and `recover_inventory_file`.
2. **Atomic Temp-File Write & Rename**:
   - Remediation for `THREAT-HREC-02`: In `recover_inventory_file`, replaced direct `fs::write` with writing to a process-isolated temporary file (`<path>.tmp.<pid>`) followed by atomic rename `fs::rename`.
   - Prevents partial write corruption, race conditions, or symlink hijacking during recovery.
3. **Hardening Unit Testing**:
   - Implemented `test_hval_hardening` in `test_hardware_recovery.rs` verifying rejection of parent directory traversal (`../evil.json`), control characters (`evil\x00store.json`), and non-json extensions (`hardware.txt`).
