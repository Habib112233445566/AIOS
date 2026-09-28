# Evidence: T-02505 Privilege Escalation Prevention Data Model Unit Tests

- **Task**: `T-02505`
- **Sub-Epic**: Privilege Escalation Prevention / data model
- **Date**: 2026-09-29
- **Status**: PASSED

## Summary of Test Coverage
1. Created `code/aiosh-rust/aiosh-core/tests/test_privilege_data_model.rs` validating:
   - `test_actor_id_validation_rules`: Actor ID presence, whitespace trimming, control character rejection, and length ceiling (`MAX_ACTOR_ID_LEN = 128`).
   - `test_capability_capacity_limit`: Capability set insertion, idempotency, and capacity bound (`MAX_CAPABILITIES_COUNT = 32`).
   - `test_privilege_dropping_boundary_enforcement`: Safe downgrade behavior, clearing of active elevation grants, capability pruning based on target tier floor, and rejection of invalid upward drops.
   - `test_elevation_grant_rules_and_kernel_tier_defense`: Grant requirement enforcement, elevation state tracking, immutable `SystemKernel` tier defense (`PRIVESC_ERR_KERNEL_TIER_IMMUTABLE`), and baseline restoration on revocation.
   - `test_transition_request_verdict_matrix`: Complete verdict transitions (`Allowed`, `GrantRequired`, `Denied`).

## Test Results
- Integration test suite `test_privilege_data_model`: 5/5 passed.
- Lib test suite `aiosh_core::privilege_data_model`: 4/4 passed.
- 0 warnings, 0 failures.
