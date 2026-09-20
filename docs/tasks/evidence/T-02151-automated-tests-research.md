# Task Evidence: T-02151 (PEP Decision Engine Automated Tests: Research)

## Overview
- **Task ID**: `T-02151`
- **Task Name**: automated tests: Research
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 6: Automated Tests Subsystem (LAUNCH)
- **Timestamp**: 2026-09-21T01:07:35+05:00
- **Status**: COMPLETED

## Objective
Establish facts, constraints, and prior art for the automated end-to-end test suite of the PEP Decision Engine, following patterns established in `test_system_update_e2e.rs`.

## Research Findings & Architectural Facts

### 1. Prior Art in Policy Engine Testing
- **XACML 3.0 Conformance Test Suite**: Requires exhaustive truth-table testing for rule combining algorithms (`deny-overrides`, `permit-overrides`, `first-applicable`) under ambiguous, empty, and contradictory rule configurations.
- **Open Policy Agent (OPA) Integration Tests**: Emphasizes pure functional evaluation isolation, fault injection against corrupt policy bundles, and strict fail-closed defaults.
- **AIOS Invariant Pattern**: Follows the `UTEST1..UTEST6` pattern, formulated here as `PEPE2E1..PEPE2E6`.

### 2. Invariant Formulations (`PEPE2E1..PEPE2E6`)
1. **`PEPE2E1` (Combining Algorithm Truth Table)**:
   - Evaluates all permutations of (Permit, Deny, Unmatched) under `DenyOverrides`, `PermitOverrides`, and `FirstApplicable`.
2. **`PEPE2E2` (Obligation Delivery & Integrity)**:
   - Validates that obligations associated with matching rules are accurately passed to `PepDecision` without dropping or corruption.
3. **`PEPE2E3` (Capacity Stress & Boundary Limits)**:
   - Verifies system behavior at scale: inserting up to 5,000 rules, asserting $O(1)$ indexed lookup latency, and rejecting rule 5,001 with `PEPSERV_ERR_CAPACITY`.
4. **`PEPE2E4` (Fault Injection & Non-Destructive Quarantine)**:
   - Simulates disk corruption (truncated JSON, invalid syntax) and validates atomic quarantine to `.bak.<timestamp>` with permission `0600` without process panic.
5. **`PEPE2E5` (Input Fuzzing & Path Traversal Defense)**:
   - Injects path traversals (`..`), null bytes, terminal escape codes, and oversized strings, verifying rejection with explicit errors.
6. **`PEPE2E6` (Cross-Surface JSON Fidelity & Persistence)**:
   - Verifies roundtrip persistence and schema fidelity across Rust core, CLI, and MCP surfaces.

## Decisions & Open Questions
- **Decision 1**: Implement the automated test suite in `code/aiosh-rust/aiosh-core/tests/test_pep_decision_e2e.rs` with RAII `TestTempDir` for clean isolation.
- **Decision 2**: Cover all 6 invariants in dedicated, independent test functions.

## Acceptance
- Facts separated from assumptions.
- No code modified in research phase.
- Ready for formal specification in `T-02152`.
