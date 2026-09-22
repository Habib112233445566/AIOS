# Task Evidence: T-02218 (Grant Lifecycle / core service: Hardening)

## 1. Scope & Execution
Applied security hardening measures identified in T-02217 to `PepGrantService` (`code/aiosh-rust/aiosh-core/src/pep_grant_service.rs`):
1. **Grant Collision / Overwrite Prevention in Attenuation**:
   - Explicitly checks `self.grants.contains_key(child_id)` in `attenuate_grant`, returning `GSVC_ERR_VALIDATION` to reject collisions or intentional grant ID hijacking.
2. **Fail-Closed Expiration Evaluation in Temporal Sweep**:
   - Hardened `sweep_expired` with pre-validation of the `now_iso` timestamp; invalid RFC 3339 timestamps return `GSVC_ERR_VALIDATION`.
   - In individual grant evaluation during sweep, unparseable or corrupted `expires_at` timestamps fail closed and immediately transition the grant to `PepGrantState::Expired`.
3. **In-Memory Store Capacity Guard on Deserialization**:
   - In `load_from_path`, enforces `service.grants.len() <= MAX_GRANTS_IN_SERVICE` (5000), preventing in-memory exhaustion attacks via crafted oversized files.
4. **Storage Path Hygiene Hardening**:
   - `validate_grant_service_path` now rejects empty/whitespace paths and paths containing NUL bytes (`\0`) or control characters.
5. **Comprehensive Hardening Unit Tests**:
   - Added `test_pep_grant_service_hardening` asserting collision rejection, invalid timestamp handling, fail-closed expiration, and path validation.

---

## 2. Test Execution Output
```
> cargo test -p aiosh-core --test test_pep_grant_service
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.03s
     Running tests\test_pep_grant_service.rs (target\debug\deps\test_pep_grant_service-d0134ba36e32d643.exe)

running 12 tests
test test_pep_grant_service_evaluation_and_usage ... ok
test test_pep_grant_service_attenuation ... ok
test test_pep_grant_service_hardening ... ok
test test_pep_grant_service_cascade_revocation ... ok
test test_pep_grant_service_issue_and_query ... ok
test test_pep_grant_service_negative_attenuation_and_eval ... ok
test test_pep_grant_service_negative_capacity_and_transitions ... ok
test test_pep_grant_service_scaffold_creation ... ok
test test_pep_grant_service_sweep_expired ... ok
test test_pep_grant_service_transition_and_indexes ... ok
test test_pep_grant_service_negative_path_and_file_checks ... ok
test test_pep_grant_service_persistence ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

---

## 3. Acceptance Confirmation
- [x] Attenuation collision checks implemented and verified.
- [x] Fail-closed expiration evaluation and timestamp validation implemented.
- [x] Storage path hygiene and store deserialization capacity checks verified.
- [x] 12/12 unit tests passing standalone with 0 compiler warnings.
