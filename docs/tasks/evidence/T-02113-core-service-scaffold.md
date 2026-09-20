# Core Service Scaffold Details: T-02113

- **Task**: T-02113 (PEP Decision Engine / core service: Scaffold)
- **Subsystem**: PEP Decision Engine Core Service
- **Files**:
  - `code/aiosh-rust/aiosh-core/src/pep_decision_service.rs`
  - `code/aiosh-rust/aiosh-core/src/lib.rs`
- **Structures Scaffolded**:
  - `PepDecisionService`: Rule management, multi-indexing, atomic disk persistence.
  - In-memory indices: `by_subject`, `by_action`.
- **Validation**: Verified via `cargo check -p aiosh-core`.
