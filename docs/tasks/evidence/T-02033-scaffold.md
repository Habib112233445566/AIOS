# T-02033: Scaffold — Capability Model MCP/API Surface

See complete scaffold documentation at [T-02033-mcp-api-surface-scaffold.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02033-mcp-api-surface-scaffold.md).

- **Interfaces**: 7 MCP tools registered in `tool_manifest()` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- **Core Extension**: Added `load_or_create` to `CapabilityService` in `code/aiosh-rust/aiosh-core/src/capability_service.rs`.
- **Status**: Compiles cleanly with zero errors (`cargo check -p aiosh-mcp`).
