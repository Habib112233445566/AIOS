# T-00446 — Documentation Index Control / MCP/API surface: Integration

## 1. Integration Scope
This task integrates the Documentation Index Control MCP tools (`aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`) with the JSON-RPC stdio server (`aiosh-mcp`), the core service (`aiosh-core::doc_index_service`), and the audit logging ring.

## 2. Integration Pathways
- **MCP Server Registration**:
  - `Server::tool_manifest()` advertises schemas for `aios.doc.index.get`, `aios.doc.check`, and `aios.doc.search`.
- **Core Library Integration**:
  - Direct invocations of `doc_index_service::build_doc_index_from_paths` and `doc_index_service::validate_doc_links`.
- **Audit Logging**:
  - Wrapped inside `dispatch::recorded_call`, creating immutable audit rows for every MCP invocation.

## 3. Verification Results
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp` -> PASS (2/2 tests)
- `python code/aiosh-mcp/tests/test_doc_mcp_smoke.py` -> PASS (5/5 tests)
