# Task Evidence: T-01998 - System Update / recovery & validation: Hardening (Sub-Epic 10)

## 1. Overview
- **Task ID**: `T-01998`
- **Sub-Epic**: 10 (System Update Recovery & Validation Subsystem)
- **Goal**: Harden the recovery and validation subsystem against symlink hijacking, memory exhaustion, dangling tempfiles, and improper slot assignments.

---

## 2. Hardened Surface & Defense Implementations

1. **Symlink Defense & Metadata Inspection**:
   - Replaced all calls to `fs::metadata()` with `fs::symlink_metadata()` in `check_update_files`, `quarantine_if_invalid`, and dangling artifact pruning.
   - Refuses to read or quarantine any symlinked state file (`is_symlink()`), preventing arbitrary file relocation or symlink hijacking attacks.
   - In staging artifact pruning, safely unlinks symlinks without following them outside the staging directory.

2. **Strict Size Limits (MAX_UPDATE_STORE_SIZE = 1 MB)**:
   - Evaluates file length via `symlink_metadata().len()` prior to calling `fs::read_to_string`.
   - Rejects or treats oversized files as corrupted, avoiding heap allocation bombs or OOM panics.

3. **Atomic File Write & Temp File Cleanup**:
   - In `recover_update_files_with_backup`, all new default state files are written to `.tmp.<pid>` and atomically renamed.
   - If either write or rename fails, the temporary file is immediately removed (`fs::remove_file(&tmp_file)`), preventing orphaned temporary files on disk.

4. **In-Memory Boot Pointer Conflict Healing**:
   - Confirms that `current_slot == target_slot` conflicts automatically reassign target to the alternate slot (`current_slot.other()`) and configure rollback slot, preventing boot loops.

---

## 3. Verification
- Cargo test: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_recovery`
- 4/4 tests passed (0.12s).
