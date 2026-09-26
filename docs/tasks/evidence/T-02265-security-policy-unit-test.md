# T-02265 Unit Test: Grant Lifecycle Security Policy

**Task:** Add focused automated tests for the security policy of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. Unit Tests Created

Created `code/aiosh-rust/aiosh-core/tests/test_pep_grant_security_policy.rs` with 7 focused test functions covering positive, negative, and boundary vectors:

| Test Name | Vector Tested | Behavior Asserted |
|---|---|---|
| `test_grant_policy_default_and_validation_bounds` | Parameter validation bounds | Rejects empty version, duration < 60s, depth 0 or > 8, capacity 0 or > 50,000 |
| `test_grant_policy_prohibited_subjects` | Blacklisted subjects | Prohibits `*anonymous*`, `*nobody*`, permits authorized subjects |
| `test_grant_policy_enforcement_modes` | Multi-mode behavior | `Enforcing` rejects invalid grants; `Permissive` and `Disabled` succeed |
| `test_grant_policy_delegation_disallowed_rights` | Right delegation restrictions | Rejects `CapabilityRight::Admin` in child grants |
| `test_grant_policy_duration_ceiling` | Lifetime limits | Rejects grants exceeding `max_grant_duration_seconds` |
| `test_grant_policy_service_integration` | `PepGrantService` integration | Service rejects prohibited subject issuance under active policy |
| `test_grant_policy_persistence_roundtrip` | Serialization & security | Atomic save/load roundtrip fidelity + path traversal (`..`) rejection |

---

## 2. Acceptance Verification
- ✅ Standalone test file created under `tests/`.
- ✅ All 7 tests pass asserting observable behavior and security boundaries.
- ✅ Negative error cases (`GRANTPOL_ERR_*`) explicitly verified.
