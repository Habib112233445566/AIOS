# T-02295: Grant Lifecycle Recovery & Validation Unit Test

## Overview
This task implements automated unit tests for the PEP Grant Store Recovery and Validation Subsystem (`tests/test_pep_grant_recovery.rs`), covering healthy invariants, error detection, cycle detection, orphan detection, attenuation violation, cascade desync, corrupt store quarantine, and auto-repair salvage operations.

## Test Suite Execution Results

```text
cargo test --test test_pep_grant_recovery
   Compiling aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.68s
     Running tests\test_pep_grant_recovery.rs (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\deps\test_pep_grant_recovery-d50a2e7051c5a29b.exe)

running 7 tests
test test_validate_attenuation_violation ... ok
test test_validate_cascade_desync ... ok
test test_recover_corrupt_json_quarantine ... ok
test test_validate_cycle_detection ... ok
test test_validate_healthy_grant_store ... ok
test test_validate_orphan_grant ... ok
test test_recover_auto_repair_orphans_and_cascade ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
```

## Coverage Breakdown
1. `test_validate_healthy_grant_store`: Validates that compliant grant hierarchies (parent + child with valid attenuation and depth) yield `is_valid: true`, 0 issues, and `healthy_grants == total_grants`.
2. `test_validate_cycle_detection`: Asserts that circular parent-child delegation chains ($A \to B \to A$) are detected and flagged with `CycleDetected` error.
3. `test_validate_orphan_grant`: Asserts that active grants pointing to non-existent parents are detected and flagged with `OrphanGrant` error.
4. `test_validate_attenuation_violation`: Asserts that child grants requesting rights not conferred by parent are detected and flagged with `AttenuationViolation`.
5. `test_validate_cascade_desync`: Asserts that revoked parents with active children are identified as `CascadeDesync`.
6. `test_recover_corrupt_json_quarantine`: Asserts that invalid/corrupt JSON stores are safely backed up to `.quarantine.<ts>.json` and an empty valid store is initialized.
7. `test_recover_auto_repair_orphans_and_cascade`: Asserts that recovery creates `.bak.<ts>` backup, revokes orphan grants, propagates cascade revocations down multi-level hierarchies, and yields a healthy, compliant post-repair store.
