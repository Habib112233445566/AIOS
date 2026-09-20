# Task Evidence: T-01954 (System Update / automated tests: Implementation)

## Summary
Implemented the minimal working behavior and end-to-end automated test suites for the AIOS System Update Mechanism across Rust core and Python MCP harnesses:
1. `code/aiosh-rust/aiosh-core/tests/test_system_update_e2e.rs`:
   - `test_utest1_clean_lifecycle_e2e`: Full update cycle from `SlotA` to `SlotB` including real filesystem artifact staging, SHA-256 calculation, slot toggling, and boot confirmation.
   - `test_utest2_payload_fault_injection_e2e`: Truncated payload and bit-flip corruption fault injection verifying `UPD_DIGEST_ERROR` / `UPD_VALIDATION_ERROR` and transition to `Failed` without slot mutation.
   - `test_utest3_boot_failure_and_rollback_e2e`: Boot failure simulation on target slot and subsequent rollback restoring active slot to `SlotA`.
   - `test_utest4_quota_and_symlink_defense_e2e`: Payload size exceeding quota rejected before disk write.
   - `test_utest5_out_of_order_state_transitions_e2e`: Illegal state transitions rejected with `UPD_STATE_ERROR`.
   - `test_utest6_cross_substrate_parity_e2e`: Status serialization parity with client expectations.
2. `code/aiosh-mcp/tests/test_system_update_e2e_smoke.py`:
   - Full lifecycle simulation, cryptographic digest mismatch fault injection, and slot rollback assertions.

## Test Results
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_e2e`: 6 passed, 0 failed in 0.04s.
- `python code/aiosh-mcp/tests/test_system_update_e2e_smoke.py`: All 3 checks passed.
