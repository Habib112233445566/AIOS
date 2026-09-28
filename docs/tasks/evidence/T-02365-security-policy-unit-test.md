# Task Evidence: T-02365 (Audit Chain Extensions / security policy: Unit Test)

## 1. Metadata
- **Task ID:** `T-02365`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Security Policy Unit Tests
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 7: Security Policy (5/10) — Unit Test

---

## 2. Test Coverage & Negative Boundary Cases
Implemented 9 comprehensive unit tests in `code/aiosh-rust/aiosh-core/src/audit_chain_policy.rs`:
1. `test_policy_default_and_validation`: Verifies default configuration satisfies all bounds and defaults to `Enforcing` mode.
2. `test_policy_prohibited_actor`: Verifies `Enforcing` mode rejects prohibited actors (`anonymous`, `guest`, `untrusted`) with `AUDITPOL_ERR_DENIED`, and `Permissive` mode issues warnings.
3. `test_policy_signature_required`: Verifies tools with sensitive prefixes (`kernel:`, `sec:`, `admin:`, `pep:`) require valid Ed25519 signatures, and permits them once signed.
4. `test_policy_bounds_validation`: Tests bounds violations: major version mismatch, description overflow, causal link lower/upper bounds ($0$ or $>64$), and inverted temporal windows (`valid_from > valid_until`).
5. `test_policy_temporal_validity`: Validates before, after, and inside active temporal validity windows.
6. `test_policy_disallow_anonymous`: Confirms empty actor or empty tool strings are rejected fail-closed.
7. `test_policy_disabled_mode`: Verifies `Disabled` mode bypasses checks safely.
8. `test_policy_causal_fanout_limit`: Tests boundary causal fanout limits and non-hex parent hash rejection.
9. `test_policy_persistence_roundtrip`: Verifies atomic disk serialization and roundtrip deserialization.

---

## 3. Test Execution Results
- Command: `cargo test -p aiosh-core --lib audit_chain_policy`
- Result: **9 passed; 0 failed; 0 ignored; finished in 0.01s**.

---

## 4. Acceptance Confirmation
- [x] Focused automated tests cover valid input, invalid input, boundary values, and primary failure modes.
- [x] Negative cases explicitly asserted.
- [x] All 9 unit tests pass in isolation.
