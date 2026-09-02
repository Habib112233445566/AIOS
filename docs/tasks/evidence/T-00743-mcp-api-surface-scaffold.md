# T-00743 — Secrets & Access Hygiene / MCP/API surface: Scaffold

## 1. Scaffold Deliverables
- Added tool schemas for `aios.secrets.scan` and `aios.secrets.check` to `Server::tool_manifest()` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- Added tool call dispatcher branches in `Server::call_tool()` for `aios.secrets.scan` and `aios.secrets.check`.
- Added test stub `test_mcp_secrets_tools_execution` in `aiosh-mcp::tests`.
- Verified compilation and test pass via `cargo test --bin aiosh-mcp`.
