# Hardening Evidence: T-02108 (PEP Decision Engine / data model: Hardening)

- **Target File**: `code/aiosh-rust/aiosh-core/src/pep_decision.rs`
- **Hardening Applied**:
  1. **Path Traversal Defense**:
     - Added mandatory `resource.contains("..")` check in `PepRequest::new()`, immediately rejecting path traversal attempts in resource URIs.
  2. **Rule Count Bounds**:
     - Defined `MAX_PEP_RULES_PER_EVALUATION = 1000`.
     - In `evaluate_rules()`, requests with $> 1000$ rules are automatically failed closed with `PepDecision::default_deny()`, preventing evaluation denial-of-service.
  3. **Obligation Limits**:
     - Verified `MAX_PEP_OBLIGATIONS = 32` enforced in `validate_invariants()`.
- **Verification**:
  - `cargo test --test test_pep_decision`: 9/9 passed in 0.44s.
- **Status**: Completed.
