# Task Evidence: T-01952 (System Update / automated tests: Specification)

## 1. Specification Overview
This document specifies the exact contract, test vectors, failure modes, and invariants for the automated test harness of the AIOS System Update Mechanism (`aiosh-core::system_update_service` and `aiosh-mcp` tools).

## 2. Invariants Enforced (UTEST1 - UTEST6)

| Invariant | Name | Description | Assertion Criteria |
|---|---|---|---|
| **UTEST1** | Full Lifecycle | Complete A/B update cycle with payload persistence and boot confirmation. | `SlotA` -> `Checking` -> `Downloading` -> `Verifying` -> `Applying` -> `ReadyToReboot` -> `confirm_boot("2.0.0")` -> `Verified` -> `Idle`, `current_slot == SlotB`. |
| **UTEST2** | Payload Fault Injection | Detection of bit-level corruption and size truncation. | Digest mismatch returns `UPD_DIGEST_ERROR`; size mismatch returns `UPD_VALIDATION_ERROR`; service enters `Failed` state; target slot is NOT switched. |
| **UTEST3** | Boot Failure & Rollback | Automatic and manual rollback semantics after staging/applying. | Boot failure triggers `rollback()`; `current_slot` reverts to `SlotA`; `target_slot` resets to `SlotB`; state returns to `Idle`. |
| **UTEST4** | Security & Quota Bounds | Rejection of payload quota overflow and symlink exploitation. | Cumulative payload $> \text{max\_payload\_bytes}$ fails with `UPD_VALIDATION_ERROR`; existing symlink at destination fails with symlink rejection error. |
| **UTEST5** | State Transition Hygiene | Rejection of out-of-order and re-entrant state transitions. | Calling `apply_update()` during `Downloading` or `stage_artifact()` during `Idle` returns `UPD_STATE_ERROR` with zero state mutation. |
| **UTEST6** | Cross-Substrate JSON Parity | Parity between Rust data structures and Python/MCP JSON responses. | Status returned by Rust matches serialization consumed by Python MCP client tools. |

## 3. Test Suites & Interfaces

### 3.1 Rust Core Integration Test (`tests/test_system_update_e2e.rs`)
- **Suite Functions**:
  1. `test_e2e_clean_lifecycle()`: Validates UTEST1 and UTEST6.
  2. `test_e2e_payload_corruption_fault_injection()`: Validates UTEST2 (bit flip and truncated data).
  3. `test_e2e_boot_failure_and_rollback()`: Validates UTEST3 (unsuccessful boot recovery).
  4. `test_e2e_quota_and_symlink_defense()`: Validates UTEST4 (quota exhaustion and symlink defense).
  5. `test_e2e_out_of_order_state_transitions()`: Validates UTEST5 (illegal state operations).

### 3.2 Python / MCP End-to-End Smoke Test (`code/aiosh-mcp/tests/test_system_update_e2e_smoke.py`)
- **Execution**: Exercises end-to-end workflows via MCP JSON-RPC protocol:
  1. Check status (initially idle, slot A).
  2. Stage manifest and valid payload via temporary directory.
  3. Verify staging and apply update (verifies transition to `ready_to_reboot` and slot switch to `slot_b`).
  4. Simulate rollback and confirm restoration to `slot_a`.
  5. Test fault injection (invalid digest manifest rejected).

## 4. Error Handling & Exit Codes
- All test failures must fail loudly with non-zero exit codes.
- Temporary test directories must be cleanly removed on test completion or tear-down.
