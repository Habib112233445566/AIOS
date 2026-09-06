# T-01333: Init & Service Supervision - MCP/API Surface: Scaffold

## Metadata
- **Task ID:** `T-01333`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface
- **Status:** Complete (Scaffold)

---

## 1. Overview & Objectives
This task verifies and documents the architectural skeleton, module wiring, tool manifest schemas, dispatch routing, and interface scaffolding for the MCP service supervision tools in `code/aiosh-rust/aiosh-mcp`.

---

## 2. Scaffolded Tool Manifest (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
The MCP server exports the formal tool descriptors in `Server::tool_manifest`:
1. `aios.service.validate`:
   - Schema: properties `name` (string), `spec` (object), `grant_id` (string); `additionalProperties: false`.
   - Documentation: "Validate service name syntax (SS1) or full ServiceSpec against SS1..SS5 invariants".
2. `aios.service.list`:
   - Schema: properties `pattern` (string), `state` (string), `startup_mode` (string), `limit` (integer), `store_path` (string), `grant_id` (string); `additionalProperties: false`.
   - Documentation: "List registered system services with optional pattern, state, or startup mode filtering".
3. `aios.service.get`:
   - Schema: required `["name"]`, properties `name` (string), `store_path` (string), `grant_id` (string); `additionalProperties: false`.
   - Documentation: "Retrieve detailed specification and runtime status of a service by name".
4. `aios.service.action`:
   - Schema: required `["name", "action"]`, properties `name` (string), `action` (string), `store_path` (string), `grant_id` (string); `additionalProperties: false`.
   - Documentation: "Execute a lifecycle action against a registered service (start, stop, restart, reload, enable, disable, mask, unmask)".
5. `aios.service.order`:
   - Schema: required `["name"]`, properties `name` (string), `store_path` (string), `grant_id` (string); `additionalProperties: false`.
   - Documentation: "Compute topological activation order for a service and its dependency graph".

---

## 3. Dispatch Wiring & Signatures
Each tool is scaffolded into `Server::call_tool` matching on the canonical string name:
```rust
match name {
    "aios.service.validate" => { ... },
    "aios.service.list"     => { ... },
    "aios.service.get"      => { ... },
    "aios.service.action"   => { ... },
    "aios.service.order"    => { ... },
    // other MCP tools
}
```
Each handler extracts typed arguments, applies input sanitization (length limits, control character checks), passes through `dispatch::recorded_call` with PEP authorization and audit ring recording, and invokes the underlying `aiosh_core::service` and `aiosh_core::service_service::ServiceStore` APIs.

---

## 4. Compilation & Verification
The project compiles cleanly with zero errors:
`cargo build --bin aiosh-mcp` succeeds.
All module exports and types are verified against `aiosh_core`.
