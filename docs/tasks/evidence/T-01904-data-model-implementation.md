# Task Evidence: T-01904 - System Update Mechanism / data model: Implementation

## 1. Overview
- **Task ID**: `T-01904`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Implement complete data model behavior, slot management, validation rules, and state machine transitions for the System Update Mechanism.

---

## 2. Implementation Details
1. **Source File**: `code/aiosh-rust/aiosh-core/src/system_update.rs`
2. **Key Capabilities**:
   - **`UpdateSlot`**:
     - `other(&self) -> Self` toggles SlotA <-> SlotB.
     - `from_str_loose` parses friendly identifiers ("slot_a", "a", "slota", "slot0", etc.).
   - **`UpdateState`**:
     - `can_transition_to` enforces strictly allowed transitions:
       - Idle -> Checking, Downloading, or Failed
       - Checking -> Downloading, Idle, or Failed
       - Downloading -> Verifying, or Failed
       - Verifying -> Applying, or Failed
       - Applying -> ReadyToReboot, or Failed
       - ReadyToReboot -> Verified, RolledBack, or Failed
       - Terminal (Verified, RolledBack, Failed) -> Idle
   - **`UpdateArtifact`**:
     - `.validate()` checks non-empty filename, no control chars, max length 256, strictly valid 64-char lowercase/uppercase ASCII hex SHA-256 digest, and size within `MAX_UPDATE_PAYLOAD_SIZE`.
   - **`UpdateManifest`**:
     - `.validate()` ensures valid `update_id`, `version`, and at least 1 artifact.
     - `.total_bytes()` calculates total payload footprint across all artifacts.
     - `.has_target()` and `.find_artifact()` query specific partition artifacts.
   - **`SystemSlotStatus`**:
     - `.new(current, version)` initializes status, setting `target_slot = current.other()`.
     - `.validate()` checks `current_slot != target_slot`.
     - `.switch_slot()` swaps `current_slot` and `target_slot`, retaining `rollback_slot`.
     - `.mark_slot_success()` updates version and successful boot flag.
   - **`SystemUpdateStatus`**:
     - `.new()` initializes in `UpdateState::Idle`.
     - `.transition(next)` verifies validity and updates state, resetting progress/error when returning to `Idle`.
     - `.set_progress(percent)` clamps to 100.
     - `.set_error(err)` switches state to `UpdateState::Failed` and records error.

---

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core` passed with exit code 0.
