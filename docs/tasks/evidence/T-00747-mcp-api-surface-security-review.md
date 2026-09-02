# T-00747 — Secrets & Access Hygiene / MCP/API surface: Security Review

## 1. Threat Model & Abuse Analysis

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **M-1** | LLM Context Window Secret Infiltration | Model responses from `aios.secrets.scan` receive only `redacted_snippet` and cryptographic `fingerprint` values, preventing raw credentials from entering LLM context. | Mitigated |
| **M-2** | MCP Path Injection / Traversal | Target paths are processed through read-only file readers with bounded byte caps. | Mitigated |
| **M-3** | Resource Flooding via Repetitive Calls | File size bounding (16 MiB cap) and fast binary exclusion prevent stdio RPC pipeline stalls. | Mitigated |
| **M-4** | Non-Repudiation & Audit Evasion | Every MCP tool execution is recorded via `dispatch::recorded_call()` with input JSON and execution results written to SQLite WAL. | Mitigated |

## 2. Policy Invariants
- **Read-Only Inspection**: Gated as safe read-only diagnostics without requiring elevated PEP tokens.
- **Fail-Closed Guarantees**: Any filesystem or parsing errors return auditable JSON-RPC error envelopes.
