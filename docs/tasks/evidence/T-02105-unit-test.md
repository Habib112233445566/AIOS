# Unit Test Evidence: T-02105 (PEP Decision Engine / data model: Unit Test)

- **Target File**: `code/aiosh-rust/aiosh-core/tests/test_pep_decision.rs`
- **Subsystem**: PEP Decision Engine Data Model
- **Execution Command**: `cargo test --test test_pep_decision`
- **Test Scenarios**:
  - `test_pep_request_valid_construction`: **PASSED** (verifies complete request construction and field integrity)
  - `test_pep_request_validation_failures`: **PASSED** (verifies rejection of empty subjects, control characters, and length bounds)
  - `test_pep_decision_invariants`: **PASSED** (verifies invariant `PEPDEC1` for Permit and Deny, and invariant failure detection)
  - `test_pep_pattern_matching`: **PASSED** (verifies exact, prefix, suffix, and wildcard `*` matching)
  - `test_pep_combining_deny_overrides`: **PASSED** (verifies `Deny` strictly overrides `Permit` under `DenyOverrides`, `PEPDEC3`)
  - `test_pep_combining_permit_overrides`: **PASSED** (verifies `Permit` overrides `Deny` under `PermitOverrides`)
  - `test_pep_combining_first_applicable`: **PASSED** (verifies rule evaluation order determines outcome)
  - `test_pep_empty_rules_fail_closed_default_deny`: **PASSED** (verifies fail-closed default deny `PEPDEC1` when no rules match across all combining algorithms)
- **Result**: 8 passed; 0 failed; finished in 0.00s. Zero warnings.
