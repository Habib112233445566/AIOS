# Task Evidence: T-02139 (MCP/API Surface: Documentation)

## Overview
- **Task ID**: `T-02139`
- **Task Name**: MCP/API surface: Documentation
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 4: MCP / API Surface
- **Timestamp**: 2026-09-21T00:54:30+05:00
- **Status**: COMPLETED

## Documentation Updates
Section 7 and Section 8 in `docs/pep_decision_engine.md` have been authored and updated:
1. **Tool Index & Details**:
   - `aios.pep.status`: Parameters, response shape, and capacity indicators.
   - `aios.pep.rule_add`: Parameter constraints, JSON-RPC 2.0 example invocation, and effect types.
   - `aios.pep.rule_list`: Filtering options by subject and action.
   - `aios.pep.rule_remove`: Removal syntax and atomic persistence.
   - `aios.pep.evaluate`: Persistent store vs. inline rule evaluation semantics, combining algorithm choices.
2. **Security & Invariants**:
   - Explicit audit logging guarantees (`dispatch::recorded_call`).
   - Path hygiene rules (`validate_pep_service_path`).
   - Default-deny fail-closed behavior.
3. **Traceability Links**:
   - Linked all evidence files for Sub-Epic 4 (`T-02131` through `T-02140`).
