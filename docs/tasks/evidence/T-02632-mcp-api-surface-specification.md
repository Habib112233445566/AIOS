# Task Evidence: T-02632 — Secrets Handling MCP/API Surface Specification

## 1. Summary
Specified the contract for the Model Context Protocol (MCP) tool surface of the Secrets Handling subsystem in [SPEC-SECRETS-MCP.md](file:///docs/SPEC-SECRETS-MCP.md):
- Defined exact JSON Schema inputs and outputs for 5 MCP tools:
  - `aios.secret.store`
  - `aios.secret.get`
  - `aios.secret.list`
  - `aios.secret.rotate`
  - `aios.secret.revoke`
- Defined explicit error structures and failure modes (`ERR_SECRET_INVALID_INPUT`, `ERR_SECRET_ACCESS_DENIED`, `ERR_SECRET_NOT_FOUND`, `ERR_SECRET_REVOKED`).
- Confirmed audit effects: all tool calls routed through `dispatch::recorded_call`, writing signed events to the audit ring.
- Established security rules SECMCP1 through SECMCP5.
