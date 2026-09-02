# T-00744 — Secrets & Access Hygiene / MCP/API surface: Implementation

## 1. Implementation Deliverables
- Implemented `aios.secrets.scan` and `aios.secrets.check` handlers in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - `aios.secrets.scan`: Executes full workspace scan or single-file scan and produces structured JSON-RPC `report` envelopes with redacted finding items.
  - `aios.secrets.check`: Fast boolean cleanliness check returning `{ "ok": true, "tool": "aios.secrets.check", "is_clean": bool, "total_findings": u32, "report": SecretScanReport }`.
  - Auditing: Dispatched through `dispatch::recorded_call()` writing one immutable audit row into the SQLite WAL ring.

## 2. Verification
- Verified passing unit test in `aiosh_mcp::tests::test_mcp_secrets_tools_execution`.
