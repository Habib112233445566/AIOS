# T-00941 — Agent Handoff Protocol / MCP/API Surface: Research

## 1. Prior Art & Architecture
- **MCP Server (`aiosh-mcp`)**: Implements JSON-RPC 2.0 tool endpoints for autonomous model interactions.
- Tools to expose:
  - `aios.handoff.list`: Query active or historical handoffs with optional filtering.
  - `aios.handoff.show`: Query single handoff details and payload.
  - `aios.handoff.initiate`: Create handoff record between agents.
  - `aios.handoff.accept`: Accept pending handoff.
  - `aios.handoff.reject`: Reject pending handoff.
  - `aios.handoff.complete`: Mark handoff as completed.
  - `aios.handoff.cancel`: Cancel pending/active handoff.
- Every tool call executes through `dispatch::recorded_call`, preserving PEP and SQLite audit invariants.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Protocol Binding | Fact | Exposes standardized JSON-RPC `tools/call` schemas. |
| Audit Compliance | Fact | Automatically records an audit row for all calls via `dispatch::recorded_call`. |
| Persistence Safety | Fact | Leverages `HandoffStore` atomic persistence and recovery. |
