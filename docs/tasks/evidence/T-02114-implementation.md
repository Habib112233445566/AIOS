# Implementation Evidence: T-02114 (PEP Decision Engine / core service: Implementation)

- **Target File**: `code/aiosh-rust/aiosh-core/src/pep_decision_service.rs`
- **Implemented Features**:
  - `PepDecisionService`:
    - In-memory registry with `add_rule`, `remove_rule`, `get_rule`, and `list_rules`.
    - Multi-index maintenance across `by_subject` and `by_action`.
    - Request evaluation: `evaluate` and `evaluate_with_algorithm`.
    - Atomic disk persistence: `save_to_path` using temporary file and atomic rename (`fs::rename`).
    - Robust loading: `load_from_path`, `load_or_create`, and `load_or_recover`.
    - Non-destructive quarantine: corrupted policy files are backed up to `<file>.bak.<timestamp>` with mode `0600` on Unix platforms.
- **Status**: Compiles cleanly with zero errors/warnings.
