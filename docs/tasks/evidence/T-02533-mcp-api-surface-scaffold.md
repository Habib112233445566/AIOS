# T-02533: Privilege Escalation Prevention MCP/API Surface Scaffold

- **Task**: `T-02533`
- **Sub-Epic**: Privilege Escalation Prevention / MCP/API surface
- **Date**: 2026-09-29
- **Status**: PASSED

## 1. Scaffold Overview
Registered 5 MCP tools and scaffolded dispatch routing in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
1. `aios.privilege.status`
2. `aios.privilege.elevate`
3. `aios.privilege.drop`
4. `aios.privilege.revoke`
5. `aios.privilege.check`

## 2. Structural Verification
- Tools registered in `tools/list` schema handler with draft-07 JSON Schema.
- Tool dispatch mapped in `Server::handle_call` via `dispatch::recorded_call`.
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml --workspace`: 0 warnings, 0 errors.
