# Task Evidence: T-02255 (Grant Lifecycle Automated Tests: Unit Test)

## 1. Metadata
- **Task ID:** `T-02255`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle Automated Tests Unit Test Verification
- **Status:** Complete
- **Date:** 2026-09-23
- **Author:** AIOS Security Architecture & Verification Team

---

## 2. Test Execution Verification

### 2.1 Isolated Automated Test Suite
```text
> cargo test -p aiosh-core --test test_pep_grant_automated -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.48s
     Running tests\test_pep_grant_automated.rs (target\debug\deps\test_pep_grant_automated-dc862e1d711e70ad.exe)

running 9 tests
test test_automated_grant_cascade_revocation_branching ... ok
test test_automated_grant_attenuation_depth_and_invariants ... ok
test test_automated_grant_concurrent_eval_and_mutation ... ok
test test_automated_grant_cli_mcp_cross_substrate ... ok
test test_automated_grant_mass_expiration_sweep ... ok
test test_automated_mock_env_initialization ... ok
test test_automated_grant_security_boundaries_and_fuzzing ... ok
test test_automated_grant_persistence_and_reload_integrity ... ok
test test_automated_grant_scale_and_indexing ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

### 2.2 Comprehensive Grant Lifecycle Suite (40 Tests Green)
```text
> cargo test -p aiosh-core --test test_pep_grant_automated --test test_pep_grant --test test_pep_grant_service --test test_pep_grant_config
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.54s

     Running tests\test_pep_grant.rs: 10 passed; 0 failed; finished in 0.02s
     Running tests\test_pep_grant_automated.rs: 9 passed; 0 failed; finished in 0.05s
     Running tests\test_pep_grant_config.rs: 9 passed; 0 failed; finished in 0.01s
     Running tests\test_pep_grant_service.rs: 12 passed; 0 failed; finished in 0.02s

Total: 40 passed; 0 failed; 0 ignored; 0 warnings.
```

### 2.3 Cross-Substrate Python Smoke Verification
```text
> python -m pytest code/aiosh-mcp/tests/test_pep_decision_smoke.py
============================== 7 passed in 1.59s ==============================
```

---

## 3. Coverage Analysis & Test Vectors

1. **Happy Path Coverage**:
   - Multi-subject grant issuance, indexing, and lookup verified up to 1,000 grants (`AUTOGRANT1`).
   - Monotonic 7-tier attenuation chain traversing root to leaf (`AUTOGRANT2`).
   - Branching DAG cascade revocation leaves unlinked branches active (`AUTOGRANT3`).
   - Mass expiration sweep identifies and transitions expired records idempotently (`AUTOGRANT4`).
   - Clean disk roundtrip serialization and index rebuilding (`AUTOGRANT5`).
   - Multithreaded concurrent evaluation under load (`AUTOGRANT7`).
   - CLI/MCP JSON schema interoperability (`AUTOGRANT8`).

2. **Negative & Adversarial Coverage (`AUTOGRANT6`)**:
   - `PEPGRANT_ERR_INVALID_ID`: Control characters, whitespace, and empty identifiers rejected.
   - `PEPGRANT_ERR_INVALID_ID`: Lengths exceeding 128 characters rejected.
   - `PEPGRANT_ERR_VALIDATION`: Subject path traversal (`agent/../../root`) rejected.
   - `PEPGRANT_ERR_VALIDATION`: Delegation depth exceeding maximum limit (> 8) rejected.
   - `PEPGRANT_ERR_VALIDATION`: Empty right sets rejected.
   - `GSVC_ERR_INVALID_TRANSITION`: Disallowed FSM transitions (e.g. `Requested -> Expired`) rejected.
   - `GSVC_ERR_NOT_FOUND`: Revocation of non-existent grants rejected.
   - `GSVC_ERR_VALIDATION`: Storage path traversal (`../traversal.json`) rejected.

---

## 4. Acceptance Criteria Checklist
- [x] New test file runs standalone and passes all 9 integration tests.
- [x] Negative and boundary cases explicitly asserted.
- [x] Zero regressions across existing test suites.
- [x] Clean compilation with zero compiler warnings.
