# T-02331: Audit Chain Extensions MCP/API Surface Research

## Objective
Establish facts, constraints, and prior art for exposing Audit Chain Extensions over the Model Context Protocol (MCP) in `aiosh-mcp`.

## Facts vs Assumptions

### Facts
1. **MCP Server Architecture**: `aiosh-mcp/src/main.rs` serves JSON-RPC 2.0 requests over stdio, registering tools in `tools/list` and handling execution in `tools/call`.
2. **Existing Audit MCP Tools**:
   - `aios.audit.tail` (properties: `n: integer`)
   - `aios.audit.verify` (properties: `full: boolean`)
   - `aios.audit.rotate` (properties: `keep_rows: integer, grant_id: string`)
   - `aios.audit.segments`
   - `aios.audit.seen` (properties: `hash: string, exact: boolean`)
3. **Audit Dispatch Protocol**: Read-only diagnostic tools record their invocations via `dispatch::recorded_call` emitting an audit row into `AuditRing` and returning the JSON tool output.
4. **Backend Readiness**: `AuditChainService` in `aiosh-core` already encapsulates `query_events`, `get_row_by_hash`, `trace_ancestry`, and `verify_event_signature`.

### Assumptions
1. AI agents operating via MCP need structured tools to discover antecedent execution context (`ancestry`), query previous actions by session/trace (`query`), verify cryptographic authenticity (`sign_verify`), and inspect detailed metadata (`inspect`).
2. Input schemas should follow standard JSON Schema with explicit types and required property lists.

## Prior Art & MCP Protocol Standards
- **Model Context Protocol (MCP) Specification (Anthropic/Open Source, 2024)**: JSON Schema 2020-12 formatted tool definitions with typed input arguments and structured content responses.
- **W3C Distributed Tracing / OpenTelemetry**: Context propagation conventions using trace IDs and span IDs.

## Decisions Needed Prior to Implementation
1. **Tool Naming Convention**:
   - `aios.audit.query`
   - `aios.audit.inspect`
   - `aios.audit.ancestry`
   - `aios.audit.sign_verify`
2. **Schema & Dispatch Integration**:
   - Register definitions in `tools/list` vector.
   - Dispatch tool calls via `dispatch::recorded_call` or `AuditChainService` directly using `self.ring`.
