# T-00844 — Regression Triage / MCP/API: Implementation

## 1. Implementation Deliverables
- Registered and implemented 5 MCP JSON-RPC 2.0 tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - `aios.triage.list`
  - `aios.triage.show`
  - `aios.triage.record`
  - `aios.triage.resolve`
  - `aios.triage.check`
- Added criterion `T4` to `tools/test_triage_suites.py`.
- Verified MCP tool flow runs and passes cleanly.
