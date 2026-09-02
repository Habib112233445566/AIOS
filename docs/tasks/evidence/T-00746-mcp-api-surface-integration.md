# T-00746 — Secrets & Access Hygiene / MCP/API surface: Integration

## 1. Integration Deliverables
- Integrated `aios.secrets.scan` and `aios.secrets.check` into the standard MCP tool manifest and stdio JSON-RPC routing table in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- Validated end-to-end JSON-RPC invocation:
  - `tools/list`: Reports `aios.secrets.scan` and `aios.secrets.check` with typed JSON schemas.
  - `tools/call`: Dispatches to `scan_file_for_secrets` / `scan_workspace_for_secrets` and logs audit events to SQLite WAL ring (`AuditRing::write`).
- Verified zero secret leakage in JSON-RPC tool call responses.

## 2. Integration Output
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "ok": true,
    "tool": "aios.secrets.check",
    "is_clean": true,
    "total_findings": 0,
    "report": { ... }
  }
}
```
