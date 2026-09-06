# T-01351: Init & Service Supervision - Automated Tests: Research

## Metadata
- **Task ID:** `T-01351`
- **Subsystem:** `code/aiosh-rust/aiosh-core`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`, `tools/`
- **Component:** Init & Service Supervision Automated Testing Architecture & Prior Art
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Research Objective

Establish facts, constraints, and prior art for the automated testing of the Init & Service Supervision subsystem across the unified Rust userspace stack, ensuring comprehensive automated regression coverage for data models, supervision engine FSM, dependency topology, CLI commands, and MCP tools.

---

## 2. Authoritative Sources & Upstream Standards

1. **systemd Integration Test Harness**:
   - Upstream: `test/test-systemctl/`, `test/units/`.
   - Core methodology: Validates state machines through deterministic sequential actions (`start` -> `stop` -> `restart`), asserts invariant rejection on masked units, validates topological boot order, and tests timeout bounds under slow or non-responsive targets.
2. **Rust Cargo Integration Testing Guide**:
   - Upstream: The Cargo Book (§Integration Tests).
   - Core methodology: Standalone test crates under `tests/` compiling against public crate interfaces (`aiosh_core`), ensuring test isolation, clean RAII resource teardown, and parallel-safe temp paths.
3. **AIOS Architecture Decisions (ADR-0035, AI Constitution P-1..P-6)**:
   - Every state transition must write an audit record to the SQLite WAL `AuditRing`.
   - Tests must assert observable side effects (store JSON on disk, audit log records, exit codes), not internal private state.

---

## 3. Analysis of Existing Test Coverage

| Test Suite | Location | Scope |
|---|---|---|
| `test_service_data_model.rs` | `code/aiosh-rust/aiosh-core/tests/` | Invariants `SS1..SS5`, syntax validation, cyclic dependency rejection in specs. |
| `test_service_service.rs` | `code/aiosh-rust/aiosh-core/tests/` | Store persistence, FSM lifecycle transitions, Kahn's algorithm topological sort (`CS1..CS5`). |
| `test_service_config.rs` | `code/aiosh-rust/aiosh-core/tests/` | Configuration resolution, precedence, and boundary invariants (`SC1..SC7`). |
| `aiosh` unit tests | `code/aiosh-rust/aiosh-cli/src/main.rs` | Operator CLI subcommands (`validate`, `list`, `show`, `action`, `order`, `config`). |
| `aiosh-mcp` unit tests | `code/aiosh-rust/aiosh-mcp/src/main.rs` | MCP tools (`aios.service.validate`, `list`, `get`, `action`, `order`, `config`). |
| `tools/test_service_suites.py` | `tools/` | Consolidated runner covering criteria `SS1..SS5`. |

---

## 4. Fact vs. Assumption

### Established Facts:
1. Each individual module in `aiosh-core` has component-level unit tests.
2. The standalone runner `tools/test_service_suites.py` runs sub-suites sequentially with strict 120s timeouts.
3. A dedicated holistic automated test suite (`tests/test_service_automated.rs`) is required for the automated tests epic (`T-01351..T-01360`), testing end-to-end integration scenarios (e.g. store corruption recovery, multi-turn dependency startup order execution, audit ring verification).

### Assumptions:
1. All automated test scenarios can run on developer machines and CI without requiring root or real PID namespace privileges by utilizing virtualized service descriptors and store files.

---

## 5. Decisions Needed Before Implementation

1. **Decision D1 (Test File Placement)**:
   Implement holistic automated tests in `code/aiosh-rust/aiosh-core/tests/test_service_automated.rs` matching the pattern established by `test_package_automated.rs`.
2. **Decision D2 (Runner Integration)**:
   Extend `tools/test_service_suites.py` with criterion `SS6` referencing the automated test suite upon completion of the epic.

---

## 6. Acceptance Criteria Verification
- [x] Authoritative sources collected and cited.
- [x] Facts separated from assumptions.
- [x] Unknowns and decisions explicitly recorded.
- [x] No code modified during research task.
