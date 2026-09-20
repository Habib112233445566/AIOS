# Task Evidence: T-02159 (PEP Decision Engine Automated Tests: Documentation)

## Overview
- **Task ID**: `T-02159`
- **Task Name**: automated tests: Documentation
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 6: Automated Tests Subsystem
- **Timestamp**: 2026-09-21T01:18:50+05:00
- **Status**: COMPLETED

## Documentation Deliverables

### 1. Master Documentation Updates
- Updated `docs/pep_decision_engine.md`:
  - **Section 10**: Authored "Automated Test Suite Reference (Sub-Epic 6)".
    - Invariants matrix `PEPE2E1..PEPE2E6` detailing test objectives and boundary assertions.
    - Copy-pasteable invocation commands for cargo test and python smoke suites.
    - Constraints and known limitations: deterministic rule ID sorting under `FirstApplicable`, and 1,000 rule evaluation bounds.
  - **Section 9**: Traceability links added for tasks `T-02151` through `T-02160`.

### 2. Copy-Pasteable Invocation Commands
```bash
# Run cargo end-to-end integration tests
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_pep_decision_e2e

# Run CLI smoke tests
python code/aiosh-cli/tests/test_pep_cli_smoke.py

# Run MCP smoke tests
python code/aiosh-mcp/tests/test_pep_decision_smoke.py
```
