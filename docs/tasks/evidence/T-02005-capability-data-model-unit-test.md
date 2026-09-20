# Task Evidence: T-02005 - Capability Model / data model: Unit Test (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02005`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Add focused automated tests for the Capability Model data model in `code/aiosh-rust/aiosh-core/tests/test_capability_data_model.rs`.

---

## 2. Test Suite Details

The unit test suite covers all 6 capability invariants:
1. **`test_cap1_creation_and_unforgeability`**:
   - Validates unforgeable ID generation with SHA-256 (`cap_<ts>_<hash>`).
   - Asserts rejection of empty issuer, empty subject, and empty rights list.
2. **`test_cap2_rights_and_scoping`**:
   - Tests right checking (`has_right`, `check_right`).
   - Tests scope containment for Filesystem (recursive vs non-recursive), Tool (allowed actions allowlist vs unauthorized actions), and System subsystems.
3. **`test_cap3_monotonic_attenuation`**:
   - Tests derivation of attenuated child capability with subset rights and narrowed subpath scope.
   - Enforces failure if child attempts to gain rights not present in parent (`InvalidAttenuation`).
   - Enforces failure if child scope exceeds parent scope.
   - Enforces failure if parent lacks `CapabilityRight::Delegate`.
4. **`test_cap4_temporal_and_quotas`**:
   - Tests expiration (`expires_at`) and premature validity (`not_before`).
   - Tests invocation quota exhaustion (`max_invocations`).
   - Tests byte quota exhaustion (`quota_bytes`).
5. **`test_cap5_revocation`**:
   - Asserts that invoking `revoke()` blocks subsequent validity checks, invocations, and byte consumption with `CapabilityError::Revoked`.
6. **`test_cap6_json_serialization_roundtrip`**:
   - Asserts exact serialization and deserialization fidelity for complex capabilities.

---

## 3. Execution Results
- Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_capability_data_model`
- Output: 6 passed; 0 failed; 0 ignored (0.00s).
