# Task Evidence: T-01995 - System Update / recovery & validation: Unit Test (Sub-Epic 10)

## 1. Overview
- **Task ID**: `T-01995`
- **Sub-Epic**: 10 (System Update Recovery & Validation Subsystem)
- **Goal**: Implement focused unit tests in `code/aiosh-rust/aiosh-core/tests/test_system_update_recovery.rs` covering invariants `UVAL1..UVAL6`.

---

## 2. Test Suite Details
The unit test suite covers:
1. `test_uval1_path_validation`: Validates path checks: length $\le 1024$, `.json` extension requirement, control character rejection, and parent directory traversal (`..`) defense.
2. `test_uval2_in_memory_validation_and_recovery`: Validates clean state diagnostics, detects slot conflict (`current_slot == target_slot`) and stuck update states, and asserts in-memory self-healing restores valid state and resolves conflicts.
3. `test_uval3_disk_check_and_quarantine_recovery`: Tests corrupt state file detection on disk, timestamped quarantine (`.corrupt.<timestamp>`), fresh state restoration, and post-recovery health validation.
4. `test_uval4_dangling_artifact_pruning`: Tests unreferenced staged artifact discovery and deletion in the staging directory, confirming reclaimed byte accounting.
