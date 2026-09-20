# Task Evidence: T-02152 (PEP Decision Engine Automated Tests: Specification)

## Overview
- **Task ID**: `T-02152`
- **Task Name**: automated tests: Specification
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 6: Automated Tests Subsystem
- **Timestamp**: 2026-09-21T01:07:55+05:00
- **Status**: COMPLETED

## Technical Specification: Automated End-to-End Test Suite

### 1. Test Harness Architecture
- **Location**: `code/aiosh-rust/aiosh-core/tests/test_pep_decision_e2e.rs`.
- **Isolation Mechanism**: RAII `TestTempDir` providing hermetic temporary filesystem sandboxing with automatic cleanup upon test completion or panic.

### 2. Test Cases & Invariant Matrix (`PEPE2E1..PEPE2E6`)

#### `test_pepe2e1_combining_algorithm_matrix`
- **Inputs**: Conflicting policy rules (one `permit`, one `deny`) matching subject `agent:worker`, action `execute`, resource `sys:kernel:module`.
- **Assertions**:
  - `DenyOverrides`: Must resolve to `effect: Deny`, `allowed: false`.
  - `PermitOverrides`: Must resolve to `effect: Permit`, `allowed: true`.
  - `FirstApplicable`: Must resolve to the effect of the rule with lowest evaluation index.
  - Unmatched Request: Must resolve to default-deny (`effect: Deny`, `allowed: false`).

#### `test_pepe2e2_obligation_delivery`
- **Inputs**: Rule configured with structured obligations (`audit_log`, `notify_admin`, `throttle`).
- **Assertions**:
  - Decision outcome includes all configured obligations in `PepDecision.obligations`.
  - Obligation types, levels, and messages match rule definitions exactly.

#### `test_pepe2e3_capacity_stress_and_boundary_limits`
- **Inputs**: Batch creation of 5,000 distinct policy rules in `PepDecisionService`.
- **Assertions**:
  - Registry successfully stores 5,000 rules (`service.rule_count() == 5000`).
  - Indexed lookup evaluates correctly.
  - 5,001st rule addition fails with `PEPSERV_ERR_CAPACITY`.

#### `test_pepe2e4_corrupt_store_fault_injection_and_quarantine`
- **Inputs**: Disk file populated with truncated/corrupted JSON bytes.
- **Assertions**:
  - `PepDecisionService::load_or_recover` safely handles error without panicking.
  - Returns `recovered == true` and `Some(quarantine_path)`.
  - Corrupt file is moved to `.bak.<timestamp>`.
  - Fresh registry is initialized with 0 rules.

#### `test_pepe2e5_input_fuzzing_and_path_traversal`
- **Inputs**: Fuzzed payloads containing `..`, null bytes, ANSI escape sequences, non-`.json` extensions, and strings exceeding length caps (> 1024 for paths, > 128 for rule IDs).
- **Assertions**:
  - All malformed inputs are rejected with explicit validation error codes.

#### `test_pepe2e6_cross_surface_persistence_and_json_parity`
- **Inputs**: Full service state with multiple indexed rules and obligations.
- **Assertions**:
  - `save_to_path` atomically writes JSON.
  - `load_from_path` restores identical rules with lossless fidelity.
