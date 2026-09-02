# T-00546 — Evidence & Audit Trail / MCP/API surface: Integration

## 1. Integration Scope
This task integrates the Model Context Protocol (MCP) tool endpoints for Evidence & Audit Trail (`aios.evidence.verify`, `aios.evidence.hash`, `aios.evidence.scan`) into `aiosh-mcp`, verifying JSON-RPC 2.0 tool discovery, schema matching, and integration with the audit ring.

## 2. Integration Points
- `tools/list` enumerates all three tools with JSON Schema definitions.
- `tools/call` executes each tool, validating parameters and writing immutable SHA-256 hash chains to SQLite WAL via `dispatch::recorded_call`.
- `code/aiosh-mcp/tests/test_evidence_mcp_smoke.py` passes all 8 test cases.

## 3. Verification
- `cargo test -p aiosh-mcp` -> PASS.
- `python code/aiosh-mcp/tests/test_evidence_mcp_smoke.py` -> PASS (8/8).
