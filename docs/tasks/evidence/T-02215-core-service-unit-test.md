# Task Evidence: T-02215 (Grant Lifecycle / core service: Unit Test)

## 1. Scope & Execution
Authored a focused, comprehensive automated unit and negative test suite for `PepGrantService` in `code/aiosh-rust/aiosh-core/tests/test_pep_grant_service.rs` covering happy path, negative inputs, boundary values, and primary failure modes:
1. **Instantiation & Empty State**: `test_pep_grant_service_scaffold_creation`
2. **Issuance & Multi-Index Querying**: `test_pep_grant_service_issue_and_query` (asserting `by_subject` and `by_state` indexing).
3. **FSM Transitions & Index Updating**: `test_pep_grant_service_transition_and_indexes` (Requested -> Active -> Suspended).
4. **Attenuation Rights Containment**: `test_pep_grant_service_attenuation` (valid child derivation vs unauthorized escalation rejection).
5. **Evaluation & Quota Tracking**: `test_pep_grant_service_evaluation_and_usage` (checking quota decrement and auto-transition to `Expired`).
6. **Transitive Cascade Revocation**: `test_pep_grant_service_cascade_revocation` (3-tier tree revocation).
7. **Temporal Expiration Sweep**: `test_pep_grant_service_sweep_expired` (identifying and transitioning past-deadline grants).
8. **Atomic Serialization & Reload**: `test_pep_grant_service_persistence` (roundtrip JSON persistence and index reconstitution).
9. **Negative Transitions & Capacity**: `test_pep_grant_service_negative_capacity_and_transitions` (non-existent grant, terminal state escape attempt).
10. **Negative Attenuation & Action Evaluation**: `test_pep_grant_service_negative_attenuation_and_eval` (ghost parent, inactive parent, subject mismatch, right mismatch).
11. **Negative Path & Storage Validation**: `test_pep_grant_service_negative_path_and_file_checks` (invalid extension, non-existent load, directory rejection).

---

## 2. Standalone Test Execution Output
```
> cargo test -p aiosh-core --test test_pep_grant_service
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.19s
     Running tests\test_pep_grant_service.rs (target\debug\deps\test_pep_grant_service-d0134ba36e32d643.exe)

running 11 tests
test test_pep_grant_service_attenuation ... ok
test test_pep_grant_service_cascade_revocation ... ok
test test_pep_grant_service_evaluation_and_usage ... ok
test test_pep_grant_service_issue_and_query ... ok
test test_pep_grant_service_negative_attenuation_and_eval ... ok
test test_pep_grant_service_negative_capacity_and_transitions ... ok
test test_pep_grant_service_negative_path_and_file_checks ... ok
test test_pep_grant_service_scaffold_creation ... ok
test test_pep_grant_service_sweep_expired ... ok
test test_pep_grant_service_transition_and_indexes ... ok
test test_pep_grant_service_persistence ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

---

## 3. Acceptance Confirmation
- [x] Test suite runs standalone in isolation and passes 100% (11/11 tests).
- [x] Negative cases and boundary limits thoroughly asserted (not just happy paths).
- [x] Observable behavior asserted across FSM transitions, errors, file persistence, and multi-index synchronization.
