# T-02415: Sandbox Enforcement Core Service Unit Test

## 1. Test Suite Summary
Created dedicated automated tests in `code/aiosh-rust/aiosh-core/tests/test_sandbox_service.rs` covering the Sandbox Enforcement Core Service (`SandboxService`) lifecycle, capability probing, process supervision, PEP authorization gating, output capping, and audit persistence.

## 2. Test Cases Covered
1. `test_service_lifecycle_and_catalog`:
   - Asserts default registration of `standard`, `strict`, and `permissive` profiles.
   - Enforces rejection of duplicate profile registration (`ERR_SANDBOX_PROFILE_EXISTS`).
   - Asserts protected default profiles cannot be deleted (`ERR_SANDBOX_CANNOT_DELETE_DEFAULT`).
   - Validates custom profile registration and removal lifecycle.
2. `test_service_validation_rejections`:
   - Verifies rejection of empty commands (`ERR_SANDBOX_EMPTY_COMMAND`).
   - Verifies rejection of path-traversal working directories (`ERR_SANDBOX_INVALID_PATH`).
3. `test_service_command_execution_success`:
   - Executes command successfully, asserting exit code 0, captured output matching expected string, duration tracking, and component status telemetry.
4. `test_service_missing_executable_failure`:
   - Exercises non-existent command execution, asserting clean failure with exit code 127 and descriptive error status without panic.
5. `test_service_pep_gating_behavior`:
   - Validates that when `enforce_pep_grants` is enabled, execution requests without `pep_grant_id` are blocked with `ERR_SANDBOX_PEP_UNAUTHORIZED`.
   - Validates authorized execution when `pep_grant_id` is supplied.
6. `test_service_audit_trail_emission`:
   - Configures service with an in-memory `AuditRing`.
   - Validates that execution emits a cryptographic audit record to SQLite table `audit_ring` with `tool="sandbox"`, valid `grant_token`, and matching row hash.
7. `test_service_output_truncation_cap`:
   - Validates enforcement of `max_output_capture_bytes` truncation ceiling.

## 3. Results
All 7 integration test cases passed with 0 compiler warnings.
