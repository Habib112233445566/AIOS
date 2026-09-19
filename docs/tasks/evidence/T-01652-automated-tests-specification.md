# T-01652: Kernel Module Management — Automated Tests: Specification

## Metadata
- **Task ID:** `T-01652`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Kernel Module Management / automated tests
- **Status:** Complete — specification.
- **Date:** 2026-09-19
- **Depends on:** `T-01651` (Research).
- **Feeds:** `T-01653` (Scaffold).
- **Artifacts:** `docs/tasks/evidence/T-01652-automated-tests-specification.md` and `docs/tasks/evidence/T-01652-spec.md`.

---

## 1. Specification Overview

This document specifies the contracts, test cases, invariants, and execution requirements for the Kernel Module Management Automated Test Suite, comprising in-tree Rust integration tests, end-to-end Python lifecycle tests, and an aggregate test orchestrator.

---

## 2. Test Invariants (AT-KM1..AT-KM5)

- **AT-KM1 (Compound State Transitions)**: Tests must exercise complete end-to-end state transitions across all operations (`blacklist`, `unblacklist`, `autoload`, `unautoload`, `options`, `preset apply`, `export`, `import`). Each step must assert both in-memory and on-disk state.
- **AT-KM2 (Scale & Density Boundaries)**: Tests must verify store stability under scaled loads (1,024 rules, 256 autoloaded modules, 64-character module names, 1,024-byte option parameters).
- **AT-KM3 (Document Ceiling Enforcement)**: Operations attempting to persist or load documents exceeding `MAX_MODULE_DOC_BYTES` (10 MiB) must fail with explicit size errors.
- **AT-KM4 (Corruption & Truncation Resilience)**: Loading malformed or truncated store JSON files must fail cleanly with structured error envelopes without panicking or modifying the file.
- **AT-KM5 (Aggregate Orchestration)**: A single orchestrator script `tools/test_kernel_module_suites.py` must run all suites (unit, CLI smoke, MCP smoke, config smoke, and automated lifecycle cases) and report unified PASS/FAIL verdicts.

---

## 3. Test Suites & File Structure

### 3.1 In-Tree Rust Integration Suite (`code/aiosh-rust/aiosh-core/tests/test_kernel_module_automated.rs`)
- `test_at_km1_compound_lifecycle`: Exercises full sequence of store operations.
- `test_at_km2_scale_limits`: Generates 1,000 rules and 200 autoload modules, verifying serialization and reload.
- `test_at_km3_document_size_ceiling`: Verifies rejection of store exceeding 10 MiB.
- `test_at_km4_corrupted_store_handling`: Tests truncated and invalid JSON recovery.

### 3.2 Python Automated Lifecycle Suite (`code/aiosh-cli/tests/test_kernel_module_automated_cases.py`)
- `test_lifecycle_compound_workflow`: End-to-end CLI workflow testing multi-subcommand sequences with `--json`.
- `test_boundary_value_cases`: Tests maximum length module names (64 chars), long options (1024 bytes), and large rule sets.
- `test_corrupt_store_cli_behavior`: Tests CLI behavior on corrupted store files (exit code 1, `LOAD_STORE_FAILED`).
- `test_concurrent_store_isolation`: Proves that independent store files under separate tempdirs do not conflict.

### 3.3 Aggregate Orchestrator (`tools/test_kernel_module_suites.py`)
- Executes the 8 kernel module test batteries:
  1. `KM1`: Data Model Unit Tests (`test_kernel_module_data_model.rs`)
  2. `KM2`: Core Service Unit Tests (`test_kernel_module_service.rs`)
  3. `KM3`: Configuration Unit Tests (`test_kernel_module_config.rs`)
  4. `KM4`: Automated In-Tree Tests (`test_kernel_module_automated.rs`)
  5. `KM5`: CLI Smoke Suite (`test_kernel_module_cli_smoke.py`)
  6. `KM6`: MCP Smoke Suite (`test_kernel_module_mcp_smoke.py`)
  7. `KM7`: Configuration Smoke Suite (`test_kernel_module_config_smoke.py`)
  8. `KM8`: Automated Lifecycle Suite (`test_kernel_module_automated_cases.py`)

---

## 4. Exit Codes & Result Assertions
- Process exit code `0` on total success.
- Process exit code `1` on any failure.
- Execution timeout of 60 seconds per individual suite.
