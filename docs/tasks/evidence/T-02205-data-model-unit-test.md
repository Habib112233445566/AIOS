# Task Evidence: T-02205 (Grant Lifecycle / data model: Unit Test)

## 1. Scope & Execution
Authored and executed a comprehensive automated unit test suite for Grant Lifecycle data models and state transitions in `code/aiosh-rust/aiosh-core/tests/test_pep_grant.rs`:
- `test_pep_grant_valid_creation_and_validation`: Validates constructor, default fields, and structural validity.
- `test_pep_grant_invalid_identifier_and_scope`: Covers empty IDs, control characters, illegal charset, path traversal (`..`) in filesystem scope, and empty rights vectors.
- `test_pep_grant_fsm_transitions`: Validates finite state machine transitions (Requested -> Active -> Suspended -> Active -> Revoked) and asserts irreversible terminal state constraints on Revoked and Expired grants.
- `test_pep_grant_temporal_and_quota_evaluation`: Tests future `not_before`, past `expires_at`, invocation quota exhaustion, and byte quota exhaustion with automatic transition to `Expired`.
- `test_pep_grant_attenuation`: Validates monotonic subset of rights, bounded delegation depth decrement, rejection of ungranted parent rights, and prohibition of delegation when `CapabilityRight::Delegate` is absent.
- `test_pep_grant_store_operations_and_cascade_revocation`: Validates multi-grant registry, subject indexing, and recursive cascade revocation across parent-child-grandchild hierarchies.
- `test_pep_grant_store_atomic_persistence`: Tests atomic disk serialization (`save_to_path`) and roundtrip deserialization (`load_from_path`).
- `test_pep_grant_action_validation`: Tests subject and right authorization enforcement.

## 2. Test Execution Output
```
cargo test -p aiosh-core --test test_pep_grant
   Compiling aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.55s
     Running tests\test_pep_grant.rs (target\debug\deps\test_pep_grant-7ee0dc01a1535122.exe)

running 8 tests
test test_pep_grant_fsm_transitions ... ok
test test_pep_grant_attenuation ... ok
test test_pep_grant_invalid_identifier_and_scope ... ok
test test_pep_grant_action_validation ... ok
test test_pep_grant_store_operations_and_cascade_revocation ... ok
test test_pep_grant_temporal_and_quota_evaluation ... ok
test test_pep_grant_valid_creation_and_validation ... ok
test test_pep_grant_store_atomic_persistence ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

## 3. Acceptance Confirmation
- [x] Dedicated test suite runs standalone and passes (8/8 tests, 100% pass rate).
- [x] Negative, boundary, and corruption cases verified.
- [x] Observable state changes, quotas, and cascade revocation asserted.
