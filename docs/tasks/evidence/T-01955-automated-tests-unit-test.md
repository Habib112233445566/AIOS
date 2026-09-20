# Task Evidence: T-01955 (System Update / automated tests: Unit Test)

## Summary
Added focused unit tests for the System Update Mechanism automated test harness covering valid input, invalid input, boundary values, and primary failure modes:
1. `test_utest1_clean_lifecycle_e2e`: Complete valid update cycle.
2. `test_utest2_payload_fault_injection_e2e`: Bit corruption and size truncation detection.
3. `test_utest3_boot_failure_and_rollback_e2e`: Rollback restoration of slot state.
4. `test_utest4_quota_and_symlink_defense_e2e`: Quota limit violation rejection.
5. `test_utest5_out_of_order_state_transitions_e2e`: State machine illegal transitions rejection.
6. `test_utest6_cross_substrate_parity_e2e`: Serialization parity.
7. `test_staging_incomplete_artifacts_rejected`: Incomplete staging rejection before verification.
8. `test_staging_undeclared_target_rejected`: Staging target not present in manifest rejection.
9. `test_quota_boundary_exact_vs_overflow`: Boundary verification at exact quota vs. quota + 1 byte.

## Test Results
- Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_e2e`
- Output: 9 passed; 0 failed; 0 ignored; finished in 0.02s
