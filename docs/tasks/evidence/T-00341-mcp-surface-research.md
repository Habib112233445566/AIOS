# T-00341 — Dependency & Toolchain Pinning / MCP/API surface: Research

## 1. Facts (from shipped code)

### 1.1 Existing MCP Server
The `aiosh-mcp` crate provides a Model Context Protocol server.
- The server reads JSON-RPC requests from standard input and writes responses to standard output.
- All consequential or logged tool calls are routed through `dispatch::recorded_call()` or `dispatch::dispatch()` to enforce PEP grants and emit audit rows.

### 1.2 Current Toolchain MCP Surface
The `aiosh-mcp/src/main.rs` file exposes exactly one tool for toolchains:
- `aios.toolchain.config.get`: Returns the active toolchain manifest by calling `ToolchainManifest::from_env()`.
- It emits an audit row (`aios.toolchain.config.get`) via `dispatch::recorded_call`.
- It is a read-only tool (requires no PEP grant).

### 1.3 Missing Surface
The actual pinning enforcement (`enforce_toolchain`) is NOT exposed to the MCP server. Agents can read the config, but they cannot invoke the host environment validation.

## 2. Assumptions
- We need to expose the `aiosh toolchain check` logic to MCP as a tool named `aios.toolchain.check`.
- It should behave exactly like the CLI: load the manifest and run `enforce_toolchain()`.
- Since it is a read-only validation check (it mutates nothing, only errors if tools don't match), it does not require a PEP grant (`requires_grant: false`).
- An audit row MUST be emitted for the check, regardless of success or failure.

## 3. Decisions Needed
1. **Should `aios.toolchain.check` take arguments?**
   - The CLI allows `--config <path>`. Should the MCP tool allow an optional `config_path` argument?
   - **Recommendation**: No. For the MCP server, the environment is static. We don't want agents arbitrarily overriding the config path to bypass enforcement. The tool should use `ToolchainManifest::from_env()`.

## 4. Citations
- Source: [aiosh-mcp/src/main.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-mcp/src/main.rs#L122-L147)
- Source: [toolchain_service.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/toolchain_service.rs)
