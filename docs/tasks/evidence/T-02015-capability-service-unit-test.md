# Task Evidence: T-02015 - Capability Model / core service: Unit Test (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02015`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Author focused unit tests for `CapabilityService` in `code/aiosh-rust/aiosh-core/tests/test_capability_service.rs`.

---

## 2. Test Suite Details

The unit test suite validates:
1. **`test_cserv1_root_issuance_and_indexing`**:
   - Asserts root capability issuance, storage indexing, and retrieval by subject.
2. **`test_cserv2_attenuation_and_lineage`**:
   - Asserts derived child capability attenuation, lineage tracking via `parent_id`, and index updates.
3. **`test_cserv3_cascade_revocation`**:
   - Builds a 3-tier hierarchy (Root -> Child -> Grandchild) and revokes the Root.
   - Asserts that all 3 capabilities are revoked transitively and excluded from subject queries.
4. **`test_cserv4_check_access_and_quotas`**:
   - Verifies `check_access` and `has_active_capability` evaluation.
   - Verifies invocation consumption through the service layer and access denial once quota is exhausted.
5. **`test_cserv5_persistence_atomic_roundtrip`**:
   - Saves registry to disk via `save_to_path` and restores via `load_from_path`, asserting exact state and index fidelity.
6. **`test_cserv6_prune_expired`**:
   - Asserts that expired leaf capabilities are pruned while active/lineage capabilities are preserved.

---

## 3. Execution Results
- Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_capability_service`
- Result: 6 passed; 0 failed; 0 ignored (0.05s).
