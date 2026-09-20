# Hardening Details: T-02108

- **Task**: T-02108 (PEP Decision Engine / data model: Hardening)
- **Subsystem**: PEP Decision Engine Data Model
- **Hardening Enhancements**:
  - `PepRequest::new`: Rejection of `..` in resource URIs (`PEP_ERR_INVALID_RESOURCE`).
  - `evaluate_rules`: Ruleset size bounded to `MAX_PEP_RULES_PER_EVALUATION` (1000 rules max), failing closed on breach.
  - `validate_invariants`: Strict checking of obligation count bounds $\le 32$.
- **Test Evidence**:
```
running 9 tests
test test_pep_combining_deny_overrides ... ok
test test_pep_combining_first_applicable ... ok
test test_pep_combining_permit_overrides ... ok
test test_pep_decision_invariants ... ok
test test_pep_empty_rules_fail_closed_default_deny ... ok
test test_pep_hardening_controls ... ok
test test_pep_pattern_matching ... ok
test test_pep_request_valid_construction ... ok
test test_pep_request_validation_failures ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
```
