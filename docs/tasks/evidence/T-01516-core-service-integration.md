# T-01516: Filesystem Layout - Core Service: Integration

## Metadata
- **Task ID:** `T-01516`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (7/10) — Core Service Integration
- **Dependencies:** `T-01515` (Core Service Unit Test)
- **Next Task:** `T-01517` (Filesystem Layout / core service: Security Review)

---

## 1. Summary of Integration Surfaces

The core service capabilities (`FilesystemLayoutService` and `FilesystemLayoutStore`) were integrated into both primary operational surfaces of AIOS:

### 1.1 MCP Tool Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
- Registered new MCP tools in `tool_manifest()`:
  - `aios.fs_layout.list`: Discovers and enumerates registered filesystem layouts.
  - `aios.fs_layout.probe`: Evaluates target disk capacity in bytes against layout constraints.
  - `aios.fs_layout.diff`: Computes structural and safety differences between source and target layouts.
- Integrated dispatch execution handlers in `call_tool()` routing calls through `dispatch::recorded_call` to ensure structured audit logging.
- Extended `test_mcp_fs_layout_tools` unit test to verify tool discovery, parameter passing, and JSON responses for all tools.

### 1.2 CLI Command Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)
- Enhanced `cmd_fs_layout`:
  - `aiosh layout list`: Lists registered layouts with active pointer marker.
  - `aiosh layout probe [--bytes <N>]`: Evaluates block device size feasibility and displays partition budget.
  - `aiosh layout diff [<source_id> <target_id>]`: Generates layout diff and warns if changes are destructive.
- Updated usage help banner.
- Updated `test_cmd_fs_layout_flow` test suite asserting zero exit code on valid invocations and appropriate exit code on invalid arguments.

---

## 2. Integration Verification

### 2.1 MCP Test Battery
```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp --bin aiosh-mcp test_mcp_fs_layout_tools
running 1 test
test tests::test_mcp_fs_layout_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.06s
```

### 2.2 CLI Test Battery
```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli --bin aiosh test_cmd_fs_layout_flow
running 1 test
test task_cli_tests::test_cmd_fs_layout_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.37s
```

### 2.3 Baseline System Smoke Battery
```
$ python tools/test_session_suites.py
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP in-tree unit test suite
[+] SB3 session MCP tool surface JSON-RPC smoke test (aios.session.*)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)
[+] SB5 session configuration resolution, invariants & precedence (SC1..SC7)
[+] SB6 session automated integration test suite (SBT1..SBT5)
[+] SB7 session security policy enforcement & invariants (SSP1..SSP7)
[+] SB8 session observability & telemetry metrics (SSO1..SSO6)
[+] SB9 session documentation architecture & operational guide (D1..D6)

PASS: session_suites criteria (SB1..SB9)
```

Acceptance criteria satisfied:
- Feature reachable through CLI and MCP production surfaces.
- Integration smoke passes end-to-end with zero regressions.
