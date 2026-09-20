# Scaffold Evidence: T-02113 (PEP Decision Engine / core service: Scaffold)

- **Target File**: `code/aiosh-rust/aiosh-core/src/pep_decision_service.rs`
- **Module Export**: Exported in `code/aiosh-rust/aiosh-core/src/lib.rs` (`pub mod pep_decision_service;`).
- **Scaffolded Types**:
  - `PepDecisionService`: Authoritative rule repository, indexer, and evaluation service.
  - Path validator: `validate_pep_service_path()`.
  - Constants: `MAX_RULES_IN_SERVICE = 5000`, `MAX_PEP_SERVICE_STORE_SIZE = 10MB`.
- **Status**: Compiles cleanly with zero errors/warnings.
