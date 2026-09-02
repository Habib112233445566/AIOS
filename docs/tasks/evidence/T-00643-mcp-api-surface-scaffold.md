# T-00643 — Repository Health / MCP/API surface: Scaffold

## 1. Scaffold Scope
This task creates the tool registration skeleton and dispatch routing for `aios.repo.health` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Scaffold Deliverables
- Declared `aios.repo.health` tool schema in `tool_manifest()`.
- Added `aios.repo.health` routing skeleton in `call_tool()`.
- Verified compilation via `cargo check --bin aiosh-mcp`.

## 3. Compilation Verification Output
```text
    Checking aiosh-mcp v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-mcp)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.21s
```
