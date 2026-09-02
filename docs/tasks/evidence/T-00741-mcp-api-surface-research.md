# T-00741 — Secrets & Access Hygiene / MCP/API surface: Research

## 1. Prior Art & In-Tree Architecture
- **In-Tree MCP Server (`aiosh-mcp`)**: Implements JSON-RPC 2.0 stdio server providing tool discovery (`tools/list`) and invocation (`tools/call`).
- **Standard Tool Execution Pattern**:
  - Registered in `Server::tool_manifest()` with typed `inputSchema`.
  - Dispatched in `Server::call_tool()` via `dispatch::recorded_call()`.
  - Every call produces an audit trail entry in the SQLite WAL ring.
- **Secrets Scanning Engine (`aiosh-core::secrets_service`)**:
  - Reuses `scan_workspace_for_secrets` and `scan_file_for_secrets`.
  - `redact_secret_value` ensures zero secret leakage across JSON-RPC responses and logs.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Tool Names | Fact | `aios.secrets.scan` and `aios.secrets.check` aligned with `aios.repo.health` and `aios.evidence.scan`. |
| Authorization | Fact | Read-only diagnostic inspection; requires no PEP grant token (`grant_id` optional). |
| Dispatch Gating | Fact | Calls route through `dispatch::recorded_call` recording actor, inputs, and results in SQLite WAL. |
| Redaction | Fact | All finding items in JSON-RPC payloads contain strictly redacted snippets and cryptographic fingerprints. |

## 3. Decisions & Contracts Needed
1. **Tool Definitions**:
   - `aios.secrets.scan`: Accepts optional `repo_path`, `file_path`, and `max_bytes`. Returns `{ "ok": bool, "tool": "aios.secrets.scan", "report": SecretScanReport }`.
   - `aios.secrets.check`: Accepts optional `repo_path`. Returns `{ "ok": bool, "tool": "aios.secrets.check", "is_clean": bool, "total_findings": u32, "report": SecretScanReport }`.
