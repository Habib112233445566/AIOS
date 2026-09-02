# T-00843 — Regression Triage / MCP/API: Scaffold

## 1. Scaffold Deliverables
- Registered MCP JSON-RPC 2.0 schemas for:
  - `aios.triage.list`
  - `aios.triage.show`
  - `aios.triage.record`
  - `aios.triage.resolve`
  - `aios.triage.check`
- Implemented handler branches in `call_tool()` within `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- Validated via `cargo test -p aiosh-mcp --bin aiosh-mcp -- test_mcp_triage_tools`.
