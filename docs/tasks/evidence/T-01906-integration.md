# Task Evidence: T-01906 - System Update Mechanism / data model: Integration

## 1. Overview
- **Task ID**: `T-01906`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Author and execute integration smoke test for System Update Mechanism data model enforcing cross-substrate JSON serialization parity and invariants `UPD1..UPD6`.

---

## 2. Integration Suite
File: `code/aiosh-cli/tests/test_system_update_smoke.py`

- `test_upd1_slot_exclusivity`: verified slot toggling and `current != target` exclusivity.
- `test_upd2_channel_and_version`: verified channel enum values and version string constraints.
- `test_upd3_artifact_and_sha256`: verified 64-char hex SHA-256 validation and artifact boundaries.
- `test_upd4_state_machine`: verified linear transitions, rollback transitions, and invalid transition rejections.
- `test_upd5_upd6_manifest_and_json_parity`: verified manifest validation and JSON roundtrip parity.

---

## 3. Results
- Status: **PASSED (5/5 tests passed, 0 failed)**
- Command: `python code/aiosh-cli/tests/test_system_update_smoke.py`
