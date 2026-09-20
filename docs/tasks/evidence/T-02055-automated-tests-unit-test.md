# Task Evidence: T-02055 (Capability Model / automated tests: Unit Test)

## Task Information
- **Task ID**: T-02055
- **Title**: Capability Model / automated tests: Unit Test
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 6: Automated Tests
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Unit Tests
1. **Test Suite**: `code/aiosh-rust/aiosh-core/tests/test_capability_automated.rs`
2. **Test Cases Verified**:
   - `test_automated_mock_env_initialization`:
     - Verifies `MockCapabilityEnv` tempdir creation and fixture pre-population.
   - `test_automated_capability_lifecycle_matrix`:
     - Multi-tier root issuance and 3-level monotonic attenuation (Tier 0 $\rightarrow$ Tier 1 $\rightarrow$ Tier 2 $\rightarrow$ Tier 3).
     - Verifies hierarchical rights check at each tier.
     - Cascade revokes Tier 1, proving Tier 1, 2, and 3 are transitively revoked while Tier 0 (Root) remains active.
   - `test_automated_capability_attenuation_invariants`:
     - Rejection of right expansion (unauthorized right).
     - Rejection of scope widening (narrower to wider path).
     - Rejection of invocation quota expansion.
     - Rejection of expiration date extension.
   - `test_automated_capability_quota_and_consumption`:
     - Invocation limit decrement and exhaustion denial (`CAP_ERROR_QUOTA_INVOCATIONS`).
     - Byte quota accumulation and limit rejection (`CAP_ERROR_QUOTA_BYTES`).
   - `test_automated_capability_persistence_and_reload`:
     - Disk save and load round-trip with index rebuilding and state fidelity.
   - `test_automated_capability_pruning_and_temporal`:
     - Proves expired leaf capabilities and expired child capabilities are pruned while non-expired parent capabilities are preserved.
   - `test_automated_capability_fault_injection`:
     - Corrupted JSON store files, non-json extensions, and path traversal rejection.

3. **Results**:
   - All 7 automated unit tests compiled and passed with zero warnings and zero failures.
