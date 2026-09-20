# Task Evidence: T-01992 - System Update / recovery & validation: Specification (Sub-Epic 10)

## 1. Overview
- **Task ID**: `T-01992`
- **Sub-Epic**: 10 (System Update Recovery & Validation Subsystem)
- **Goal**: Specify the exact data contracts, error types, recovery actions, and verification mechanisms for `system_update_recovery.rs`.

---

## 2. Specification & Data Contracts

### 2.1 Error Codes & Constants
- `MAX_UPDATE_STORE_SIZE`: `1_048_576` (1 MB maximum file size).
- `UVAL_PATH_ERROR`: Path validation errors (traversal, length, control characters).
- `UVAL_IO_ERROR`: Filesystem I/O failures during read/quarantine/write.
- `UVAL_VALIDATION_ERROR`: Integrity or semantic constraint violations.
- `UVAL_PARSE_ERROR`: JSON parsing/syntax errors.

### 2.2 Path Validation Function
```rust
pub fn validate_update_store_path(path: &Path) -> Result<(), String>
```
Validates:
- Non-empty UTF-8.
- Length $\le 1024$ bytes.
- Zero control characters (`c.is_control() || c == '\0'`).
- Zero parent directory traversal (`Component::ParentDir`).
- File extension must be `.json` (case-insensitive).

### 2.3 `SystemUpdateValidationReport`
```rust
pub struct SystemUpdateValidationReport {
    pub state_dir: String,
    pub slot_status_valid: bool,
    pub update_status_valid: bool,
    pub staging_dir_valid: bool,
    pub slot_conflict_detected: bool,
    pub dangling_artifacts: Vec<String>,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}
```

### 2.4 `SystemUpdateRecoveryAction`
```rust
pub enum SystemUpdateRecoveryAction {
    QuarantinedCorruptFile { original_path: String, quarantine_path: String },
    RestoredDefaultSlotStatus { active_slot: UpdateSlot, version: String },
    ResetFailedUpdateState { previous_state: UpdateState },
    PrunedDanglingArtifacts { count: usize, bytes_freed: u64 },
    SynchronizedBootSlotPointer { current_slot: UpdateSlot, target_slot: UpdateSlot },
}
```

### 2.5 `SystemUpdateRecoveryReport`
```rust
pub struct SystemUpdateRecoveryReport {
    pub state_dir: String,
    pub recovered: bool,
    pub actions_taken: Vec<SystemUpdateRecoveryAction>,
    pub timestamp: String,
}
```

### 2.6 Core Functions
1. `validate_update_state(slot_status: &SystemSlotStatus, update_status: &SystemUpdateStatus, staging_dir: &Path) -> SystemUpdateValidationReport`
2. `check_update_files(state_dir: &Path, staging_dir: &Path) -> SystemUpdateValidationReport`
3. `recover_update_state_in_memory(slot_status: &mut SystemSlotStatus, update_status: &mut SystemUpdateStatus, staging_dir: &Path) -> SystemUpdateRecoveryReport`
4. `recover_update_files_with_backup(state_dir: &Path, staging_dir: &Path, fallback_version: &str, fallback_slot: UpdateSlot) -> Result<SystemUpdateRecoveryReport, String>`

---

## 3. Operational Invariants (`UVAL1..UVAL6`)
- `UVAL1`: Comprehensive state integrity checking.
- `UVAL2`: Non-destructive quarantine (`.corrupt.<timestamp>`).
- `UVAL3`: Dangling artifact pruning from staging directories.
- `UVAL4`: Safe state machine transition reset to `Idle`.
- `UVAL5`: Dual-slot coherence ensuring `target_slot = current_slot.other()`.
- `UVAL6`: Path hygiene, 1 MB file bounds, and atomic persistence.
