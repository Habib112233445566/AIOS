# Task Evidence: T-02014 - Capability Model / core service: Implementation (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02014`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Implement the complete working behavior of `CapabilityService` in `code/aiosh-rust/aiosh-core/src/capability_service.rs`.

---

## 2. Implemented Features & Invariants

1. **Registry & Secondary Indexing (`CSERV1`)**:
   - `capabilities: HashMap<String, Capability>` for $O(1)$ ID lookups.
   - `by_subject: HashMap<String, HashSet<String>>` for subject-level access queries.
   - `by_parent: HashMap<String, HashSet<String>>` for lineage tracking and cascade revocation.
2. **Root Capability Issuance (`CSERV2`)**:
   - `issue_root_capability()` validates parameters and indexes newly generated root tokens.
3. **Atomic Attenuation & Lineage (`CSERV3`)**:
   - `attenuate_capability()` validates the parent capability in the registry, enforces monotonic attenuation, binds `parent_id`, and indexes both parent and child.
4. **Cascade Transitive Revocation (`CSERV4`)**:
   - `revoke_capability()` revokes the target capability and iteratively traverses `by_parent` to revoke all transitive child capabilities, returning all affected IDs.
5. **Access Verification & Quotas**:
   - `check_access()` and `has_active_capability()` evaluate active capabilities for a subject against a required scope and right.
   - `consume_invocation_on_capability()` and `consume_bytes_on_capability()` mutate state within registry.
6. **Atomic Persistence & Bounded Storage (`CSERV5`)**:
   - `save_to_path()` and `load_from_path()` enforce symlink refusal, 10 MB ceiling, and `.tmp.<pid>` atomic replacement.
7. **Pruning & Garbage Collection (`CSERV6`)**:
   - `prune_expired()` removes expired capabilities that have no children, cleaning all secondary index sets.

---

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core`: Passed in 11.47s with zero warnings.
