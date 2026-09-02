# T-00947 — Agent Handoff Protocol / MCP/API Surface: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-1: JSON-RPC Payload Flooding & Memory Exhaustion
- **Threat**: Malicious subagent attempts to submit oversized JSON payloads or unbounded context summaries via `aios.handoff.initiate`.
- **Mitigation**: Bounds enforced on string inputs and structured serialization; store capacity bounded at 10,000 entries.

### AS-2: PEP Bypass on Model Tool Invocation
- **Threat**: Direct invocation of handoff state-changing tools without PEP authorization or audit logging.
- **Mitigation**: All MCP tool endpoints route through `dispatch::recorded_call`, evaluating PEP permissions and logging audit rows directly to the SQLite WAL before returning.

### AS-3: Store Path Traversal
- **Threat**: Attacker supplies a malicious `store_path` parameter (e.g. `../../etc/shadow`).
- **Mitigation**: HandoffStore operations are restricted to valid JSON state files within designated workspace paths.
