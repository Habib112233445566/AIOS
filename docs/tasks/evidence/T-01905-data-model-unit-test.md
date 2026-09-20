# Task Evidence: T-01905 - System Update Mechanism / data model: Unit Test

## 1. Overview
- **Task ID**: `T-01905`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Write and execute comprehensive unit tests for the System Update Mechanism data model in `aiosh-core`.

---

## 2. Test Cases Executed
File: `code/aiosh-rust/aiosh-core/tests/test_system_update.rs`

1. `test_upd1_slot_toggle_and_parsing`:
   - Validates `.other()` toggles between `SlotA` and `SlotB`.
   - Validates `from_str_loose()` parsing of aliases ("slot_a", "A", "slota", "slot0", "slot_b", "B", "slotb", "slot1") and rejection of empty/invalid strings.
2. `test_upd1_slot_status_lifecycle`:
   - Validates initial slot status creation where target slot is automatically set to `current.other()`.
   - Validates slot switching (`switch_slot()`) and rollback slot preservation.
   - Validates marking slot update success (`mark_slot_success()`).
   - Validates invariant check returning `UPD_SLOT_ERROR` when `current_slot == target_slot`.
3. `test_upd2_channel_parsing_and_serde`:
   - Validates string aliases for channels (stable, beta, nightly, dev).
   - Validates JSON serde roundtrip parity (`"beta"`).
4. `test_upd3_artifact_validation`:
   - Validates valid artifact acceptance with 64-char hex SHA-256 and positive byte size.
   - Validates rejection of empty filenames, filenames with control characters, non-64-length digests, non-hex characters in digests (`UPD_DIGEST_ERROR`), zero sizes, and sizes exceeding `MAX_UPDATE_PAYLOAD_SIZE`.
5. `test_upd3_manifest_validation_and_helpers`:
   - Validates full update manifest acceptance.
   - Validates rejection of empty/oversized `update_id` and `version`, as well as empty artifact lists.
   - Validates helpers: `total_bytes()`, `has_target()`, and `find_artifact()`.
6. `test_upd4_state_machine_transitions`:
   - Validates linear execution flow: `Idle -> Checking -> Downloading -> Verifying -> Applying -> ReadyToReboot -> Verified -> Idle`.
   - Validates rollback flow: `ReadyToReboot -> RolledBack -> Idle`.
   - Validates failure transitions from active states and error recording.
   - Validates rejection of invalid transitions (`Idle -> ReadyToReboot` returning `UPD_STATE_ERROR`).
7. `test_upd6_status_serde_parity`:
   - Validates JSON serialization and roundtrip deserialization parity for `SystemUpdateStatus`.

---

## 3. Test Output
```
running 7 tests
test test_upd1_slot_status_lifecycle ... ok
test test_upd1_slot_toggle_and_parsing ... ok
test test_upd3_artifact_validation ... ok
test test_upd2_channel_parsing_and_serde ... ok
test test_upd3_manifest_validation_and_helpers ... ok
test test_upd4_state_machine_transitions ... ok
test test_upd6_status_serde_parity ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
