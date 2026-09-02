# T-00316 — Dependency & Toolchain Pinning: Data Model Integration

## Overview
We integrated the Dependency & Toolchain Pinning `ToolchainManifest` into the AIOS MCP Server (`aiosh-mcp`), exposing the baseline configuration for agent awareness.

## Integration Details
- **Tool Registration**: Registered a new MCP tool `aios.toolchain.config.get` in `aiosh-mcp/src/main.rs`.
- **Dispatcher Wire-up**: Handled the execution of `aios.toolchain.config.get` by calling `aiosh_core::toolchain_config::ToolchainManifest::from_env()` and wrapping the output natively via `to_json_with_sources()`.
- **Audit Integration**: Reused the `dispatch::recorded_call` logic so that querying the toolchain configuration correctly logs the query to the active `AuditRing` ledger, proving the system's cross-substrate parity.

## Verification
- MCP tool definition schema accurately reports `aios.toolchain.config.get`.
- Compiled `aiosh-mcp` successfully with the new endpoints.
- Integration tests in `aiosh-mcp` (via `cargo test -p aiosh-mcp`) verify the structural handler behavior without failing.
