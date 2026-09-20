# Task Evidence: T-01915 - System Update Mechanism / core service: Unit Test

## 1. Overview
- **Task ID**: `T-01915`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Write and execute comprehensive unit tests for `SystemUpdateService` in `aiosh-core`.

---

## 2. Test Cases Executed
File: `code/aiosh-rust/aiosh-core/tests/test_system_update_service.rs`

1. `test_usvc1_initialization_and_defaults`:
   - Validates proper initialization of default slot tracking (`SlotA` active, `SlotB` target, `SlotA` rollback) and idle update status.
2. `test_usvc2_happy_path_update_lifecycle`:
   - Validates end-to-end flow: manifest intake, rootfs staging, kernel staging, digest validation, staging completeness verification, slot application (`SlotB` next boot), and post-boot confirmation with version elevation.
3. `test_usvc2_digest_mismatch_fails_and_halts`:
   - Simulates corrupted/tampered payload bytes, verifies cryptographic hash mismatch triggers `UPD_DIGEST_ERROR` and halts in `Failed` state.
4. `test_usvc2_size_mismatch_rejected`:
   - Verifies payload size differing from manifest declaration is rejected with `UPD_VALIDATION_ERROR`.
5. `test_usvc3_incomplete_staging_cannot_verify`:
   - Staging one artifact while omitting another prevents advancing to `Verifying` state.
6. `test_usvc4_atomic_state_persistence_and_reload`:
   - Atomically saves slot and update state files to disk (`.tmp` + rename pattern) and successfully reloads state into a new service instance.
7. `test_usvc5_rollback_orchestration`:
   - Tests simulated boot failure after update application, triggering `rollback()` to restore the previous functional slot (`SlotA`).
8. `test_usvc6_clean_staging`:
   - Verifies staging directory cleanup purges residual files while preserving directory existence.

---

## 3. Results
- Status: **PASSED (8/8 passed, 0 failed)**
- Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_service`
