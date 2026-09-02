# T-00841 — Regression Triage / MCP/API: Research

## 1. Prior Art & Subsystem Objectives
- **Context & Goal**:
  - `Regression Triage / MCP/API (T-00841..T-00850)` provides MCP JSON-RPC 2.0 tools for managing regressions programmatically.
  - Tools to expose:
    - `aios.triage.list`: List triage records with optional status/severity filtering.
    - `aios.triage.show`: Inspect single record details.
    - `aios.triage.record`: Record a regression finding into the store.
    - `aios.triage.resolve`: Mark a triage item as resolved with notes.
    - `aios.triage.check`: Perform boolean cleanliness check on open blocker/critical regressions.
- **Dispatch Contract**:
  - All calls route through `dispatch::recorded_call` emitting an audit row in the SQLite WAL audit ring.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| JSON-RPC 2.0 Manifest | Fact | Defined in `aiosh_mcp::Server::tool_manifest`. |
| Audit Row Logging | Fact | Every MCP tool execution writes an immutable row via `dispatch::recorded_call`. |
| Tool Parameter Schema | Fact | Explicit JSON schemas with typed properties and optional fields. |

## 3. Decisions & Contracts Needed
1. Specify tool schemas in `docs/tasks/evidence/T-00842-mcp-api-specification.md`.
2. Add tool definitions and handler branches to `code/aiosh-rust/aiosh-mcp/src/main.rs`.
3. Add criterion `T4` to `tools/test_triage_suites.py`.
