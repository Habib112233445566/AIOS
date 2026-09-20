# Data Model Unit Test Details: T-02105

- **Task**: T-02105 (PEP Decision Engine / data model: Unit Test)
- **Suite**: `aiosh-core/tests/test_pep_decision.rs`
- **Output**:
```
running 8 tests
test test_pep_decision_invariants ... ok
test test_pep_combining_permit_overrides ... ok
test test_pep_combining_first_applicable ... ok
test test_pep_combining_deny_overrides ... ok
test test_pep_empty_rules_fail_closed_default_deny ... ok
test test_pep_pattern_matching ... ok
test test_pep_request_valid_construction ... ok
test test_pep_request_validation_failures ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
- **Invariants Verified**:
  - `PEPDEC1`: Complete mediation & fail-closed default deny
  - `PEPDEC2`: Canonical request context validation
  - `PEPDEC3`: Deterministic combining algorithms (`DenyOverrides`, `PermitOverrides`, `FirstApplicable`)
  - `PEPDEC4`: Atomic decision response with obligations
  - `PEPDEC5`: Pure, side-effect-free evaluation
  - `PEPDEC6`: Auditability and cryptographic traceability
