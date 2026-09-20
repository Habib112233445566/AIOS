# Task Evidence: T-01906 - System Update Mechanism / data model: Integration

## 1. Overview
- **Task ID**: `T-01906`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Author and execute integration smoke test for System Update Mechanism data model enforcing cross-substrate JSON serialization parity and invariants `UPD1..UPD6`.

---

## 2. Integration Suite
File: `code/aiosh-cli/tests/test_system_update_smoke.py`

### Test Scenarios
1. `test_upd1_slot_exclusivity`:
   - Dual slot toggle (`slot_a` <-> `slot_b`).
   - Slot status validity and rejection when `current_slot == target_slot`.
2. `test_upd2_channel_and_version`:
   - Channel enum bounds (`stable`, `beta`, `nightly`, `development`) and rejection of unknown channels.
   - Version string max length (64 chars).
3. `test_upd3_artifact_and_sha256`:
   - Artifact schema validation, partition target enforcement, 64-character hex SHA-256 validation, payload size boundaries (positive and $\le 10$ GB).
4. `test_upd4_state_machine`:
   - Linear execution path (`idle -> checking -> downloading -> verifying -> applying -> ready_to_reboot -> verified -> idle`).
   - Rollback path (`ready_to_reboot -> rolled_back -> idle`).
   - Failure transitions from active operations.
   - Rejection of invalid leaps (`idle -> ready_to_reboot`, `downloading -> verified`).
5. `test_upd5_upd6_manifest_and_json_parity`:
   - Full manifest validation with multiple partition artifacts.
   - Python <-> Rust canonical JSON schema roundtrip parity.

---

## 3. Test Output
```
Starting System Update Mechanism Data Model Smoke Suite (UPD1..UPD6)...
PASS: test_upd1_slot_exclusivity
PASS: test_upd2_channel_and_version
PASS: test_upd3_artifact_and_sha256
PASS: test_upd4_state_machine
PASS: test_upd5_upd6_manifest_and_json_parity
ALL 5 SYSTEM UPDATE MECHANISM DATA MODEL INTEGRATION TESTS PASSED.
```
