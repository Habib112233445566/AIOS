# Task T-02495 Evidence: Sandbox Recovery & Validation Unit Testing

## Goal
Add comprehensive automated unit tests for Sandbox Enforcement Recovery & Validation.

## Test Matrix Evaluated
1. `test_sandbox_recovery_healthy_default`: Default factory state validates with `is_healthy == true`, 3 valid profiles, 0 corrupt profiles, 0 issues.
2. `test_sandbox_recovery_missing_factory_profile_and_restore`: An uninitialized service reports missing factory profiles (`SANDBOXRECV_ERR_MISSING_FACTORY`), and `RestoreFactoryDefaults` restores all 3 profiles.
3. `test_sandbox_recovery_corrupt_manifest_quarantine`: Malformed JSON in a custom profile file is flagged as corrupt (`SANDBOXRECV_ERR_CORRUPT`), and `QuarantineAndReset` atomically isolates it into `.quarantine_<timestamp>`.
4. `test_sandbox_recovery_traversal_detection`: Paths with `..` are caught and rejected fail-closed (`SANDBOXRECV_ERR_TRAVERSAL`).
5. `test_sandbox_recovery_oversized_file`: Files exceeding `MAX_PROFILE_FILE_BYTES` (64 KiB) are rejected without loading.
6. `test_sandbox_recovery_dry_run_strategy`: Dry run returns health verdict without mutating in-memory profiles or disk files.
7. `test_sandbox_recovery_serialization`: Validates serde JSON serialization and deserialization for `SandboxValidationReport`.

## Execution Results
```
running 7 tests
test test_sandbox_recovery_dry_run_strategy ... ok
test test_sandbox_recovery_healthy_default ... ok
test test_sandbox_recovery_missing_factory_profile_and_restore ... ok
test test_sandbox_recovery_traversal_detection ... ok
test test_sandbox_recovery_oversized_file ... ok
test test_sandbox_recovery_serialization ... ok
test test_sandbox_recovery_corrupt_manifest_quarantine ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```
