# Data Model Specification: T-02102

- **Task**: T-02102 (PEP Decision Engine / data model: Specification)
- **Subsystem**: PEP Decision Engine
- **Types Specified**:
  - `PepRequest`: Immutable policy request tuple.
  - `PepEnvironmentContext`: Environmental context attributes.
  - `PepDecisionEffect`: `Permit`, `Deny`, `Indeterminate`, `NotApplicable`.
  - `PepDecision`: Comprehensive decision outcome with obligations and audit trail.
  - `PepObligation`: Post-decision actions (AuditLog, RateLimit, RedactFields, Custom).
  - `PepCombiningAlgorithm`: `DenyOverrides`, `PermitOverrides`, `FirstApplicable`.
- **Validation Constraints**: Max subject len 256, max resource len 1024, max action len 64, max reason len 512.
- **Status**: Formally specified.
