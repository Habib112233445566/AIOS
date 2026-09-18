# T-01506: Filesystem Layout - Data Model: Integration

## Metadata
- **Task ID:** `T-01506`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout (`aiosh-cli` & `aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (6/10) — Data Model Integration
- **Dependencies:** `T-01505` (Unit Test)
- **Next Task:** `T-01507` (Security Review)

---

## 1. Executive Summary & Deliverables

Task `T-01506` integrated the Filesystem Layout data model across both operational production surfaces of the AIOS stack: the command-line interface (`aiosh-cli`) and the Model Context Protocol daemon (`aiosh-mcp`).

### 1.1 CLI Integration (`code/aiosh-rust/aiosh-cli/src/main.rs`)
Wired command `aiosh layout` (alias: `aiosh fs-layout`) with full audit classification (`classify_and_emit`) and JSON envelope support:
- `aiosh layout show [--standard|--container|--spec <file_or_json>] [--json]`: Displays partition and mount hierarchy.
- `aiosh layout validate [--standard|--container|--spec <file_or_json>] [--json]`: Validates layouts against invariants `FL1..FL5`.
- `aiosh layout fstab [--standard|--container|--spec <file_or_json>] [--json]`: Formats layout specifications into standard 6-field `/etc/fstab` lines.
- `aiosh layout check [--standard|--container|--spec <file_or_json>] [--json]`: Returns exit code 0 for valid, 1 for invalid.

### 1.2 MCP Surface Integration (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
Registered and implemented 3 MCP tools with typed schemas and recorded audit dispatch:
1. `aios.fs_layout.get`: Retrieves standard reference layouts (`standard_uefi` or `minimal_container`).
2. `aios.fs_layout.validate`: Validates layout specifications from file paths, raw JSON strings, or inline JSON objects against `FL1..FL5`.
3. `aios.fs_layout.fstab`: Generates `/etc/fstab` contents from layout specifications.

---

## 2. Integration Smoke & Automated Test Verification

### 2.1 CLI Integration Test (`aiosh-cli`)
Executed `cargo test -p aiosh-cli --bin aiosh test_cmd_fs_layout_flow`:
```
running 1 test
test task_cli_tests::test_cmd_fs_layout_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.33s
```

### 2.2 MCP Integration Test (`aiosh-mcp`)
Executed `cargo test -p aiosh-mcp --bin aiosh-mcp test_mcp_fs_layout_tools`:
```
running 1 test
test tests::test_mcp_fs_layout_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.10s
```

All acceptance criteria satisfied:
- [x] Feature reachable through production CLI (`aiosh layout`) and MCP (`aios.fs_layout.*`).
- [x] Integration smoke tests pass end-to-end with zero errors.
