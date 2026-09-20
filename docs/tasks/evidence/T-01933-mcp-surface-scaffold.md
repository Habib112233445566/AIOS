# Task Evidence: T-01933 - System Update / MCP/API surface: Scaffold

- **Task**: `T-01933`
- **Sub-Epic**: `Sub-Epic 4: Model Context Protocol (MCP) & API Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Scaffolded the System Update MCP tools and server interface in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
- Registered all 6 tools in `tool_manifest()`:
  - `aios.update.status`
  - `aios.update.slots`
  - `aios.update.check`
  - `aios.update.apply`
  - `aios.update.confirm`
  - `aios.update.rollback`
- Scaffolded `resolve_update_service` helper with path hygiene verification.
- Scaffolded JSON-RPC `call_tool()` routing arms for each update tool.
- Verified compilation via `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp`.
