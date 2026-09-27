# T-02333: Audit Chain Extensions MCP/API Surface Scaffold

## Overview
This task scaffolds the Model Context Protocol (MCP) surface for Audit Chain Extensions in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## Scaffolded Elements

### 1. `tools/list` Registration
Added JSON Schema definitions for four new audit extension tools:
- `aios.audit.query`: Filter extended events by `session_id`, `trace_id`, `actor`, `tool`, `parent_hash`, and `limit`.
- `aios.audit.inspect`: Fetch detailed metadata for an audit event by required `hash`.
- `aios.audit.ancestry`: Trace execution DAG upwards by required `hash` and optional `depth`.
- `aios.audit.sign_verify`: Cryptographically verify public key digital signature for required `hash`.

### 2. `call_tool` Routing & Handlers
Implemented dispatch handlers wrapping `AuditChainService` via `dispatch::recorded_call` in `aiosh-mcp/src/main.rs`:
- Ensures full audit row logging of every MCP invocation.
- Converts typed results to standard JSON tool response envelopes.

### 3. Build & Compilation Verification
Verified with `cargo check -p aiosh-mcp` compiling cleanly with 0 errors and 0 warnings.
