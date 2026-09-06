# T-01331: Init & Service Supervision - MCP/API Surface: Research

## Metadata
- **Task ID:** `T-01331`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface
- **Status:** Complete (Research)

---

## 1. Executive Summary & Objectives
The goal of this task is to establish facts, constraints, security invariants, and prior art for the Model Context Protocol (MCP) API surface of Init & Service Supervision in AIOS. We analyze the existing MCP tool implementation in `code/aiosh-rust/aiosh-mcp`, compare against industry standard service management RPC protocols (systemd D-Bus, Android init IPC), and evaluate compliance with AIOS governance (ADR-0035, ADR-0036).

---

## 2. Existing Codebase & Surface Analysis

### 2.1 Tool Manifest & Schemas (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
The MCP server exports the following tools in `Server::tool_manifest`:
1. `aios.service.validate`:
   - Purpose: Validates service name syntax (`SS1`) or a full `ServiceSpec` object against `SS1..SS5` invariants.
   - Input Schema:
     - `name`: `string` (optional, 1..128 chars, alphanumeric + dashes/underscores/dots, ending in `.service`).
     - `spec`: `object` (optional, full `ServiceSpec` representation).
     - `grant_id`: `string` (optional PEP capability token).
   - Invariant: Exactly one of `name` or `spec` must be provided. Rejects control characters.
2. `aios.service.list`:
   - Purpose: Lists registered services with structured filtering (name/description substring pattern, state, startup mode, limit).
   - Input Schema:
     - `pattern`: `string` (optional, max 256 chars, no control chars).
     - `state`: `string` (optional enum: `active`, `inactive`, `activating`, `deactivating`, `failed`, `reloading`).
     - `startup_mode`: `string` (optional enum: `enabled`, `disabled`, `static`, `masked`).
     - `limit`: `integer` (optional positive integer limit).
     - `store_path`: `string` (optional custom JSON store path, max 1024 chars, no control chars).
     - `grant_id`: `string` (optional PEP capability token).
3. `aios.service.get`:
   - Purpose: Retrieves detailed specification and runtime status of a service.
   - Input Schema:
     - `name`: `string` (required, canonical service name).
     - `store_path`: `string` (optional custom JSON store path).
     - `grant_id`: `string` (optional PEP capability token).
4. `aios.service.action`:
   - Purpose: Executes a lifecycle state transition or startup mode alteration.
   - Input Schema:
     - `name`: `string` (required).
     - `action`: `string` (required enum: `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`).
     - `store_path`: `string` (optional custom JSON store path).
     - `grant_id`: `string` (optional PEP capability token).
   - Side Effects: Atomically persists state to store if `store_path` is specified.
5. `aios.service.order`:
   - Purpose: Computes topological startup sequence using Kahn's algorithm.
   - Input Schema:
     - `name`: `string` (required).
     - `store_path`: `string` (optional custom JSON store path).
     - `grant_id`: `string` (optional PEP capability token).

### 2.2 Dispatch, Authorization & Audit Pipeline
- Every service MCP tool routes through `dispatch::recorded_call`.
- `dispatch::recorded_call` executes:
  1. PEP authorization check (`check_authorization`).
  2. Classification and audit log record generation via `classify_and_emit`.
  3. Safe closure execution with structured `Result<Value, String>`.
  4. Packaging into standard JSON envelope: `{ "ok": bool, "tool": string, ... }` or `{ "ok": false, "error": string, "code": string }`.

---

## 3. Prior Art & Authoritative Sources

| Standard / System | Interface / Mechanism | Key Lessons & AIOS Mapping |
|:------------------|:----------------------|:---------------------------|
| **Model Context Protocol (MCP)** (Anthropic, Nov 2024) | JSON-RPC 2.0 over stdio/HTTP; `tools/list`, `tools/call`. Strict schema validation via JSON Schema Draft 7. | AIOS tools implement explicit input schemas with `additionalProperties: false`, standard parameter descriptions, and structured JSON results. |
| **systemd D-Bus API** (`org.freedesktop.systemd1`) | D-Bus methods: `GetUnit`, `ListUnits`, `StartUnit`, `StopUnit`, `ReloadUnit`, `EnableUnitFiles`, `MaskUnitFiles`. | Granular unit inspection, separation between specification and dynamic unit status, explicit job/result objects. |
| **s6 / s6-rc** (Skarnet) | Compiled dependency graph database, atomic transitions between service sets. | Validation of complete dependency closure, acyclicity checks before attempting state transitions. |
| **ADR-0035 / ADR-0036** | Mandatory audit logging, capability tokens (`grant_id`), non-repudiation, tamper-evident audit ring. | Consequential actions (e.g. `aios.service.action`) emit audit events with target service name and actor context. |

---

## 4. Fact vs Assumption Matrix

### Facts (Empirically Verified in Code)
1. **Tool Existence**: All five service tools (`aios.service.validate`, `list`, `get`, `action`, `order`) are registered in `aiosh-mcp`'s `tool_manifest`.
2. **Execution Hook**: All five tools are integrated into `Server::call_tool` dispatch with `dispatch::recorded_call`.
3. **Audit Emission**: Invocations write audit entries to `self.ring` via `classify_and_emit`.
4. **Input Constraints**:
   - `name`: length <= 128, no ASCII control characters.
   - `pattern`: length <= 256, no ASCII control characters.
   - `store_path`: length <= 1024, no ASCII control characters.
   - `spec`: validated against `SS1..SS5` via `aiosh_core::service::validate_service_spec`.
5. **Atomic Persistence**: `ServiceStore::save_to_path` writes to a PID-isolated temporary file (`.tmp.<pid>`) with fsync and rename, removing temp files on failure.
6. **Existing Test Coverage**: `code/aiosh-rust/aiosh-mcp/src/main.rs` contains `test_mcp_service_tools` asserting 12 scenarios covering valid/invalid inputs, filtering, actions, and topological order.

### Assumptions (To Verify & Validate in Epics)
1. **JSON Payload Size Limits**: In JSON-RPC over stdio, framed messages could theoretically be unbounded unless capped by the transport reader. (Transport layer framing in `aiosh-mcp` must enforce maximum message size, e.g. 1 MiB or 2 MiB).
2. **Concurrent File Access**: If multiple processes invoke MCP tools against the same `store_path` simultaneously, atomic rename prevents file corruption but last-writer-wins may overwrite concurrent edits.
3. **Error Consistency**: All errors returned by MCP tools must provide structured error strings and avoid panics or uncaught exceptions.

---

## 5. Decisions Needed Before Implementation

1. **Decision 1: Tool Surface Completeness**
   - *Status*: DECIDED.
   - *Rationale*: The 5 tools (`validate`, `list`, `get`, `action`, `order`) provide 100% functional parity with the CLI surface (`aiosh service`) and cover all lifecycle management operations required by LLM agents. No extraneous tools needed.
2. **Decision 2: Verification Suite Alignment**
   - *Status*: DECIDED.
   - *Rationale*: In `tools/test_service_suites.py`, criterion `SS3` is titled `"service MCP tool surface (validate)"`. It should be updated to `"service MCP tool surface (validate, list, get, action, order)"` to accurately reflect the comprehensive test suite executed by `test_mcp_service_tools`.
3. **Decision 3: Standalone Python MCP Smoke Test**
   - *Status*: DECIDED.
   - *Rationale*: While Rust unit tests (`test_mcp_service_tools`) test the in-process server, a Python smoke test (`code/aiosh-mcp/tests/test_service_mcp_smoke.py` or similar) can verify JSON-RPC stdio protocol interaction end-to-end.

---

## 6. Research Conclusion
The MCP service supervision surface is solidly architected, adheres strictly to ADR-0035/ADR-0036 audit and capability requirements, and cleanly maps to `aiosh-core::service_service`. The next specification step (`T-01332`) will formally document the JSON-RPC interface schemas, envelope semantics, and error codes.
