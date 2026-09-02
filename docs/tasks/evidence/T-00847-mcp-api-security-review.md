# T-00847 — Regression Triage / MCP/API: Security Review

## 1. Threat Model & MCP API Security Analysis

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **TRG-MCP-1** | Unlogged Mutation Attempt | Modifying MCP tools (`record`, `resolve`) route through `dispatch::recorded_call`, guaranteeing immutable audit row emission. | Mitigated |
| **TRG-MCP-2** | Memory Exhaustion via Malicious Store Payload | Bounded store file reading (1 MiB cap) and bounded string fields (`MAX_ERROR_MSG_BYTES`, etc.) prevent memory exhaustion. | Mitigated |
| **TRG-MCP-3** | Insecure Param Injection | JSON input properties are strongly typed and parsed without shell invocation. | Mitigated |

## 2. Invariant Checklist
- [x] Zero unlogged mutations.
- [x] Fail-closed validation for missing parameters.
- [x] Bounded serialization and memory footprint.
