# T-00943 — Agent Handoff Protocol / MCP/API Surface: Scaffold

## 1. Scaffold Deliverables
- Added tool manifests in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - `aios.handoff.list`
  - `aios.handoff.show`
  - `aios.handoff.initiate`
  - `aios.handoff.accept`
  - `aios.handoff.reject`
  - `aios.handoff.complete`
  - `aios.handoff.cancel`
- Wired dispatch routing in `call_tool` invoking `dispatch::recorded_call`.
- Project builds and checks cleanly with 0 errors.
