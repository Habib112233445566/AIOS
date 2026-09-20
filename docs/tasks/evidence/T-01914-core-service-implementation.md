# Task Evidence: T-01914 - System Update Mechanism / core service: Implementation

## 1. Overview
- **Task ID**: `T-01914`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Implement complete update orchestration, artifact staging, cryptographic hash verification, A/B slot switching, boot confirmation, rollback, and state persistence in `aiosh-core`.

---

## 2. Implementation Details
File: `code/aiosh-rust/aiosh-core/src/system_update_service.rs`

Key features:
1. **Manifest Validation & Intake (`check_manifest`)**:
   - Validates manifest schema and artifact count bounds.
   - Enforces idle state entry.
   - Transitions `Idle -> Checking -> Downloading`.
   - Initializes staging directory.
2. **Artifact Staging & Cryptographic Verification (`stage_artifact`)**:
   - Verifies target exists in manifest.
   - Verifies exact byte length matches declaration.
   - Computes SHA-256 digest on the fly via `Sha256` and matches case-insensitively.
   - Fails and halts on mismatch with `UPD_DIGEST_ERROR`.
   - Saves artifact to isolated staging path.
   - Updates progress percentage incrementally.
3. **Staging Completeness Gate (`verify_staged`)**:
   - Verifies all declared partition targets have been staged.
   - Transitions `Downloading -> Verifying`.
4. **Boot Slot Application (`apply_update`)**:
   - Transitions `Verifying -> Applying`.
   - Toggles inactive partition to active (`slot_status.switch_slot()`).
   - Transitions `Applying -> ReadyToReboot`.
5. **Boot Confirmation & Rollback (`confirm_boot`, `rollback`)**:
   - `confirm_boot` marks target slot as booted successfully and returns system to `Idle`.
   - `rollback` restores partition pointer from `rollback_slot` and returns system to `Idle`.
6. **Atomic Persistence & Recovery (`save_state_to_dir`, `load_state_from_dir`)**:
   - Writes `slot_status.json.tmp` and `update_status.json.tmp` and atomically renames.
   - Loads persisted status back into memory cleanly.

---

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core` passed with zero errors and zero warnings.
