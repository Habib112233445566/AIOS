# T-01833: Network Bootstrap / MCP/API Surface: Scaffold

## 1. Overview
- **Task ID**: `T-01833`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Scaffold the MCP tool schemas and call dispatch for Network Bootstrap in `aiosh-mcp`.

---

## 2. Changes Made
1. **Tool Schema Declarations in `code/aiosh-rust/aiosh-mcp/src/main.rs`**:
   - Added 7 tools to `tool_manifest`:
     - `aios.network.list`
     - `aios.network.show`
     - `aios.network.routes`
     - `aios.network.dns`
     - `aios.network.state`
     - `aios.network.up`
     - `aios.network.down`
   - Configured input schemas with path parameters (`sysfs_path`, `procfs_path`, `resolv_path`) and `grant_id`.
2. **Dispatch Match Arms**:
   - Wired tool name routing in `call_tool`.
   - Scaffolded `resolve_network_service` helper for hermetic path resolution.
3. **Compilation**:
   - `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp` succeeded with 0 errors.
