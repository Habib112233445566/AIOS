# Task Evidence: T-01991 - System Update / recovery & validation: Research (Sub-Epic 10)

## 1. Overview
- **Task ID**: `T-01991`
- **Sub-Epic**: 10 (System Update Recovery & Validation Subsystem)
- **Goal**: Research and establish architecture, invariants, and authoritative sources for health checking, validation, and automated corruption recovery for the AIOS System Update Mechanism.

---

## 2. Research & Prior Art in AIOS
- Analyzed existing AIOS recovery modules: `network_recovery.rs`, `session_recovery.rs`, `service_recovery.rs`, and `hardware_recovery.rs`.
- Key architecture patterns:
  - Strongly-typed `SystemUpdateValidationReport` and `SystemUpdateRecoveryReport`.
  - Non-destructive automated self-healing with timestamped quarantine (`.corrupt.<timestamp>`).
  - Staging directory dangling artifact pruning.
  - Safe in-memory and on-disk recovery procedures (`recover_update_state_in_memory`, `recover_update_files_with_backup`).
  - Strict path hygiene (`validate_update_store_path`) and bounded file I/O ($\le 1$ MB).

---

## 3. Fact vs. Assumption Matrix

| Aspect | Fact | Assumption / Decision |
|---|---|---|
| State File Corruption | Power loss or abrupt reboots during update check or write can truncate JSON state | Implement automated detection and timestamped quarantine before re-initializing fresh state |
| Staging Directory Leaks | Interrupted downloads leave partial or orphaned payload files on disk | Implement automated dangling artifact pruning comparing staging files against active manifest |
| Slot Conflict Incoherence | Broken state could report `current_slot == target_slot` or corrupted version strings | Self-healing must enforce `target_slot = current_slot.other()` and ensure bootable fallback |
| Safety & Non-Destruction | Erasing state files without backup risks unrecoverable bootloader state loss | Always preserve corrupted files as timestamped backups before writing replacements |

---

## 4. Invariants Formulated (`UVAL1..UVAL6`)
1. **`UVAL1` (Comprehensive State Validation)**: Validates slot status integrity (`current_slot != target_slot`, version format, boot success flags), update status, and staging directory existence.
2. **`UVAL2` (Automated Non-Destructive Quarantine)**: Corrupt state files are renamed with `.corrupt.<timestamp>` suffix prior to replacing with fresh state.
3. **`UVAL3` (Dangling Artifact Pruning)**: Staging directory files not referenced in the active manifest are pruned to reclaim quota space.
4. **`UVAL4` (State Machine Self-Healing)**: Unrecoverable intermediate states (e.g. `Downloading` without active manifest or stuck in `Applying`) are safely transitioned back to `Idle`.
5. **`UVAL5` (Dual-Slot Coherence)**: Guarantees active slot and target slot are distinct and that `rollback_slot` points to a viable partition.
6. **`UVAL6` (Path & Persistence Hygiene)**: Path validation rejects `..`, control characters, non-JSON extensions, and sizes $> 1$ MB, using atomic `.tmp.<pid>` persistence.
