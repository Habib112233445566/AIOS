# T-01326: Init & Service Supervision - CLI Surface: Integration

## Metadata
- **Task ID:** `T-01326`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Integration
- **Status:** Complete

## 1. Production Surfaces Integrated
- **Root Dispatcher (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
  - Wired `Some("service") => cmd_service(&args[1..])` into top-level command routing.
  - Updated root usage documentation string:
    `aiosh service <validate|list|show|status|action|start|stop|restart|reload|order>  Init & Service Supervision Control`
- **Subcommands & Action Shortcuts**:
  - `validate`: Invariant validation for names (`SS1`) and full specifications (`SS1..SS5`).
  - `list`: Multi-parameter service querying (`--pattern`, `--state`, `--mode`, `--limit`, `--store`, `--json`).
  - `show` & `status`: Service specification and runtime state inspection.
  - `action`: State machine transition dispatcher (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`).
  - Direct Action Shortcuts: `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask` routing directly with option preservation.
  - `order`: Topological startup execution calculation with cycle detection via Kahn's algorithm.

## 2. Cross-Substrate Parity & Audit
- **Deterministic JSON Envelopes**:
  - In `--json` mode, all subcommands return standard envelopes `{"code": <exit_code>, "data": ..., "error": ...}` matching API expectations.
- **Synchronous Audit Logging**:
  - Every invocation branch (success and failure) calls `classify_and_emit`, appending immutable audit records to `audit.log` / SQLite WAL ring (`audit.db`).
- **Atomic Persistence**:
  - Operations modifying store state (`action`, `start`, `stop`, etc.) atomically serialize and rename temporary files (`.tmp.<pid>`), preventing store corruption.

## 3. Automated Test Verification
- **Rust Integration Suite (`task_cli_tests::test_cmd_service_flow`)**:
  - 100% passing coverage for all subcommands, shortcuts, exit codes, and error conditions.
- **Standalone Smoke Suite (`code/aiosh-cli/tests/test_service_cli_smoke.py`)**:
  - Verifies compiled binary execution end-to-end against live temporary stores.
- **Master Test Runner (`tools/test_service_suites.py`)**:
```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```
