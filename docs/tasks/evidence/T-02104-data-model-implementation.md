# Data Model Implementation Details: T-02104

- **Task**: T-02104 (PEP Decision Engine / data model: Implementation)
- **Subsystem**: PEP Decision Engine
- **Core Functions**:
  - `PepRequest::new`: Input validation and unique request ID generation.
  - `PepDecision::permit`: Explicit permit outcome.
  - `PepDecision::deny`: Explicit deny outcome.
  - `PepDecision::default_deny`: Fail-closed default deny outcome (`PEPDEC1`).
  - `PepPolicyRule::matches`: Pattern and wildcard matching across subject, resource, action.
  - `evaluate_rules`: Deterministic evaluation with combining algorithms (`PEPDEC3`).
  - `PepDecision::validate_invariants`: Self-validation of `PEPDEC1..PEPDEC6` invariants.
