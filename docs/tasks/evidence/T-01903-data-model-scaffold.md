# Task Evidence: T-01903 - System Update Mechanism / data model: Scaffold

## 1. Overview
- **Task ID**: `T-01903`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Scaffold core structures, enums, constants, and validation methods for the System Update Mechanism in `aiosh-core`.

---

## 2. Scaffolded Files & Exports
1. **Module Source**: `code/aiosh-rust/aiosh-core/src/system_update.rs`
   - Constants:
     - `MAX_UPDATE_VERSION_LEN: usize = 64`
     - `MAX_UPDATE_ID_LEN: usize = 128`
     - `MAX_UPDATE_PAYLOAD_SIZE: u64 = 10 * 1024 * 1024 * 1024` (10 GB)
     - `UPD_VALIDATION_ERROR`, `UPD_SLOT_ERROR`, `UPD_DIGEST_ERROR`, `UPD_STATE_ERROR`
   - Enums:
     - `UpdateSlot` (`SlotA`, `SlotB`) with `.other()`, `.as_str()`, and `.from_str_loose()`
     - `UpdateChannel` (`Stable`, `Beta`, `Nightly`, `Development`) with `.as_str()`, and `.from_str_loose()`
     - `UpdateState` (`Idle`, `Checking`, `Downloading`, `Verifying`, `Applying`, `ReadyToReboot`, `Verified`, `RolledBack`, `Failed`) with `.can_transition_to()` implementing `UPD4`
     - `PartitionTarget` (`Rootfs`, `Kernel`, `Initramfs`, `FullBundle`)
   - Structs:
     - `UpdateArtifact`: `target`, `file_name`, `sha256`, `size_bytes` with `.validate()` (validates 64-char hex SHA-256 digest and payload limits)
     - `UpdateManifest`: `update_id`, `version`, `channel`, `min_version`, `artifacts`, `signature`, `release_notes`, `published_at` with `.validate()`
     - `SystemSlotStatus`: `current_slot`, `target_slot`, `rollback_slot`, `slot_a_version`, `slot_b_version`, `slot_a_successful`, `slot_b_successful` with `.new()` and `.validate()`
     - `SystemUpdateStatus`: `state`, `current_version`, `target_version`, `active_slot`, `progress_percent`, `last_error`, `updated_at`
2. **Module Registration**: `code/aiosh-rust/aiosh-core/src/lib.rs`
   - `pub mod system_update;`
   - `pub use system_update::{...};`

---

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core` compiled with exit code 0.
