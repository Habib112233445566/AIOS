# Task Evidence: T-01916 - System Update Mechanism / core service: Integration

## 1. Overview
- **Task ID**: `T-01916`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Author and execute cross-surface Python integration smoke tests for `SystemUpdateService` enforcing invariants `USVC1..USVC6`.

---

## 2. Integration Test Scenarios
File: `code/aiosh-cli/tests/test_system_update_service_smoke.py`

1. `test_usvc1_staging_isolation`:
   - Validates that staging directory and state directories are created cleanly and operate in isolated sandboxed paths.
2. `test_usvc2_cryptographic_verification`:
   - Validates that legitimate payload matching declared SHA-256 is staged successfully.
   - Validates that tampered payload with mismatched hash fails with `UPD_DIGEST_ERROR` and halts in `failed` state.
3. `test_usvc3_active_slot_non_interference`:
   - Validates that active slot (`slot_a`) is unaffected during download and staging; update application targets the alternate slot (`slot_b`), and `rollback_slot` retains the previous slot.
4. `test_usvc4_atomic_persistence`:
   - Validates that `slot_status.json` and `update_status.json` are written atomically without leaving orphaned `.tmp` files.
5. `test_usvc5_usvc6_lifecycle_and_rollback`:
   - Validates lifecycle transitions (`idle -> downloading -> verifying -> applying -> ready_to_reboot`).
   - Validates rollback returning system cleanly to active slot `slot_a` in `idle` state.
   - Validates post-boot confirmation marking `slot_b` as successful with updated version.

---

## 3. Results
- Status: **PASSED (5/5 tests passed, 0 failed in 0.10s)**
- Command: `python code/aiosh-cli/tests/test_system_update_service_smoke.py`
