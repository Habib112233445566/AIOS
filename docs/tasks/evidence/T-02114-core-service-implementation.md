# Core Service Implementation Details: T-02114

- **Task**: T-02114 (PEP Decision Engine / core service: Implementation)
- **Subsystem**: PEP Decision Engine Core Service
- **Functions Implemented**:
  - `PepDecisionService::add_rule`: Capacity enforcement and multi-index update (`by_subject`, `by_action`).
  - `PepDecisionService::remove_rule`: Removal and index cleanup.
  - `PepDecisionService::evaluate`: Request evaluation with default or explicit combining algorithm.
  - `PepDecisionService::save_to_path`: Atomic file persistence with symlink rejection and parent directory creation.
  - `PepDecisionService::load_or_recover`: Non-destructive quarantine of corrupted policy files to `.bak.<timestamp>` with mode 0600.
