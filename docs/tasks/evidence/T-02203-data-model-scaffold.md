# Task Evidence: T-02203 (Grant Lifecycle / data model: Scaffold)

## 1. Scope & Execution
Created the module skeleton, core data models, and error definitions for Grant Lifecycle in `code/aiosh-rust/aiosh-core/src/pep_grant.rs`:
- Defined `PepGrantState` (Requested, Active, Suspended, Revoked, Expired).
- Defined `PepGrantRevocation` context.
- Defined `PepGrantConstraints` (temporal bounds, invocation quota, byte quota, delegation depth).
- Defined `PepGrant` entity with lifecycle validation, state machine transition checks, attenuation, and usage recording.
- Defined `PEPGRANT_ERR_*` error constants.
- Registered module in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Created test stub `code/aiosh-rust/aiosh-core/tests/test_pep_grant.rs` exercising grant creation and validation.

## 2. Build & Test Verification
```
cargo test -p aiosh-core --test test_pep_grant
     Running tests\test_pep_grant.rs (target\debug\deps\test_pep_grant-7ee0dc01a1535122.exe)

running 1 test
test test_pep_grant_scaffold_creation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 3. Acceptance Confirmation
- [x] Project builds with zero errors.
- [x] New interfaces exist and are referenced by test stub.
- [x] Invariants PEPGRANT1..PEPGRANT6 scaffolded cleanly.
