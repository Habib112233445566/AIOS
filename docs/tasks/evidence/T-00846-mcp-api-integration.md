# T-00846 — Regression Triage / MCP/API: Integration

## 1. Integration Deliverables
- Integrated all 5 triage MCP tools (`aios.triage.list`, `aios.triage.show`, `aios.triage.record`, `aios.triage.resolve`, `aios.triage.check`) into `Server::tool_manifest` and `Server::call_tool`.
- Routed execution through `dispatch::recorded_call`, writing immutable audit rows to the SQLite WAL audit ring.
- Verified end-to-end MCP tool discovery and execution via `tools/test_triage_suites.py` criterion T4.
