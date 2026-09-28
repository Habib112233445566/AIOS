# Evidence: T-02515 Privilege Escalation Prevention Core Service Unit Tests

- **Task**: `T-02515`
- **Sub-Epic**: Privilege Escalation Prevention / core service
- **Date**: 2026-09-29
- **Status**: PASSED

## Summary of Test Coverage
1. Created `code/aiosh-rust/aiosh-core/tests/test_privilege_service.rs`:
   - `test_service_capacity_and_bounds`: Verifies capacity ceiling enforcement (`PRIVESC_ERR_CAPACITY_EXCEEDED`), capacity configuration, and `clear()`.
   - `test_service_actor_not_found_errors`: Verifies unknown actor rejection (`PRIVESC_ERR_ACTOR_NOT_FOUND`) across queries, unregister, drop, revoke, and elevation.
   - `test_service_multi_step_escalation_and_dropping`: Verifies multi-stage monotonic elevation with capabilities, capability pruning on drop, and baseline level restoration on revocation.
   - `test_service_kernel_tier_defense`: Verifies strict rejection of `SystemKernel` elevation attempts from userspace.
   - `test_service_duplicate_actor_handling`: Verifies prevention of duplicate actor context registration (`PRIVESC_ERR_CONTEXT_EXISTS`).
2. Confirmed 5/5 tests in `test_privilege_service` passed, 2/2 tests in `privilege_service` library passed.
3. 0 warnings, 0 errors.
