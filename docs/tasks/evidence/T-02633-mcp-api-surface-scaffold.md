# Task Evidence: T-02633 — Secrets Handling MCP/API Surface Scaffold

## 1. Summary
Created the scaffold and module wiring for Secrets Handling over the MCP protocol in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
- Defined vault path and safe loader helpers: `get_secrets_store_path()` and `load_safe_secret_service()`.
- Added 5 new tools to `Server::tool_manifest`:
  - `aios.secret.store`
  - `aios.secret.get`
  - `aios.secret.list`
  - `aios.secret.rotate`
  - `aios.secret.revoke`
- Registered tool execution dispatch arms in `Server::call_tool` routing through `dispatch::recorded_call`.

## 2. Compilation Verification
- `cargo check -p aiosh-mcp`: Compiled cleanly in 9.49s with 0 errors, 0 warnings.
- `cargo check --workspace`: Clean build across all crates.
