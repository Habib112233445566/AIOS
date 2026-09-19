# T-01651: Kernel Module Management — Automated Tests: Research

## Metadata
- **Task ID:** `T-01651`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Kernel Module Management / automated tests
- **Status:** Complete — research established prior art, test inventory, coverage gaps, and decisions.
- **Date:** 2026-09-19
- **Depends on:** `T-01650` (Configuration Verification & Evidence)
- **Feeds:** `T-01652` (Automated Tests Specification)
- **Artifacts:** `docs/tasks/evidence/T-01651-automated-tests-research.md` and `docs/tasks/evidence/T-01651-research.md`

---

## 1. Existing Test Inventory & Prior Art

The Kernel Module Management subsystem currently possesses:
1. **In-Tree Rust Unit Tests (`code/aiosh-rust/`)**:
   - `aiosh-core/tests/test_kernel_module_data_model.rs`: Validates `ModprobeRule`, `ModuleInfo`, `KernelModulePreset`, and invariants KM1..KM5.
   - `aiosh-core/tests/test_kernel_module_service.rs`: Validates `KernelModuleService`, procfs fallback, pre-commit conflict detection, atomic persistence, and canonical presets.
   - `aiosh-core/tests/test_kernel_module_config.rs`: Validates `KernelModuleManagementConfig`, modprobe.d and modules-load.d line parsers, file ingestion, and CFG-KM1..CFG-KM5.
   - `aiosh-cli/src/main.rs::test_cmd_kernel_module_flow`: In-tree CLI flow test.
   - `aiosh-mcp/src/main.rs::test_mcp_kernel_module_tools`: In-tree MCP tool dispatch test.
2. **External Integration Test Suites (`code/aiosh-cli/tests/` & `code/aiosh-mcp/tests/`)**:
   - `test_kernel_module_cli_smoke.py`: Operator CLI command dispatch, JSON envelopes, and terminal escape filtering.
   - `test_kernel_module_mcp_smoke.py`: JSON-RPC protocol compliance, all 10 `aios.kernel_module.*` tools, and cross-surface parity.
   - `test_kernel_module_config_smoke.py`: File import/export integration, conflict detection, and roundtrip fidelity.

---

## 2. Coverage Gaps & Objectives for Automated Tests Sub-Epic

While individual commands and tools have isolated unit and smoke tests, the following integration and stress areas require dedicated automated test coverage:
1. **End-to-End Multi-Step Lifecycle Transitions**:
   - Sequential compound flows: initializing store -> blacklisting legacy drivers -> applying hardened security preset -> exporting to modprobe.d/modules-load.d -> importing into a secondary store -> asserting exact state equivalence.
2. **Boundary & Stress Limits**:
   - Testing store document size ceiling (10 MiB limit `MAX_MODULE_DOC_BYTES`).
   - Testing rule count limits (up to 1,024 rules) and autoload limits (up to 256 modules).
   - Testing module name length limits (exactly 64 characters) and parameter value lengths (1024 bytes).
3. **Store Corruption Recovery & Failure Modes**:
   - Truncated or malformed JSON stores must fail safely without crash or partial overwrite.
   - Read-only store paths must surface clear domain errors (`SAVE_STORE_FAILED`).
4. **Unified Orchestration**:
   - An aggregate test runner `tools/test_kernel_module_suites.py` to coordinate all unit, CLI smoke, MCP smoke, and automated test batteries in a single invocation.

---

## 3. Fact vs. Assumption Analysis

| Topic | Established Fact | Assumption / Working Hypothesis |
|---|---|---|
| **Test Isolation** | Shared store files between tests cause race conditions and test interference. | Each automated test case must run with a dedicated `tempfile.TemporaryDirectory` and distinct store path. |
| **Cross-Platform Execution** | Windows and Linux handle subprocess path separators and exit statuses differently. | All tests must use `pathlib.Path` abstractions and capture stdout/stderr via bounded timeouts. |
| **Error Envelopes** | Both CLI and MCP emit structured envelopes (`code`, `data`, `error`). | Automated assertions should validate specific structured error codes rather than free-form error text. |

---

## 4. Decisions Needed for Specification (T-01652)

- **D1: Automated test suite language and placement**:
  *Decision:* Implement both in-tree Rust integration tests (`test_kernel_module_automated.rs`) and Python end-to-end test harness (`test_kernel_module_automated_cases.py`).
- **D2: Scope of aggregate runner**:
  *Decision:* Implement `tools/test_kernel_module_suites.py` coordinating the 5 kernel module suites: Core Data Model, Core Service, Core Config, CLI Smoke, MCP Smoke, and Automated Cases.
- **D3: Failure mode assertions**:
  *Decision:* Assert exact exit codes (0 for success, 1 for domain/conflict error, 2 for invocation/syntax error) and structured error codes.

---

## 5. Acceptance Confirmation
- [x] Evidence file exists and separates facts from assumptions.
- [x] No code changed; decisions needed are listed explicitly.
