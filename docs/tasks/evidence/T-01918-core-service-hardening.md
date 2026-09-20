# Task Evidence: T-01918 - System Update Mechanism / core service: Hardening

## 1. Overview
- **Task ID**: `T-01918`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Implement security hardening mitigations for `SystemUpdateService` in `aiosh-core`.

---

## 2. Hardening Measures Implemented
File: `code/aiosh-rust/aiosh-core/src/system_update_service.rs`

1. **Symlink Hijack Defense (`THREAT-USVC-01`)**:
   - In `stage_artifact()`, checks `fs::symlink_metadata()` on destination file. If it exists and is a symlink, rejects with `UPD_VALIDATION_ERROR` ("target file is a symbolic link (symlink attack rejected)").
2. **Accumulated Payload Quota Enforcement (`THREAT-USVC-02`)**:
   - Prior to staging each artifact, computes cumulative size across all already-staged artifacts and adds incoming artifact size. If sum exceeds `config.max_payload_bytes`, rejects with `UPD_VALIDATION_ERROR`.
3. **State Deserialization Validation (`THREAT-USVC-06`)**:
   - In `load_state_from_dir()`, invokes `slot_status.validate()?` immediately after parsing JSON, preventing corrupted or tampered slot configurations (`current_slot == target_slot`) from being loaded into memory.
4. **Stale Temporary File Hygiene (`THREAT-USVC-03`)**:
   - In `save_state_to_dir()`, actively removes pre-existing `.tmp` files (`slot_status.json.tmp`, `update_status.json.tmp`) before writing fresh state files.

---

## 3. Verification
- 10 unit tests in `test_system_update_service.rs` passed in 0.05s.
- 5 integration smoke tests in `test_system_update_service_smoke.py` passed in 0.10s.
