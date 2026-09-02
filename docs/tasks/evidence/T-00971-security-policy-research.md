# T-00971 — Agent Handoff Protocol / Security Policy: Research

## 1. Prior Art & Architecture
- Security model aligns with AIOS Policy Enforcement Point (PEP) and Principle of Least Privilege.
- Key security controls for Agent Handoff:
  - **Caller Identity Verification**: Handshake operations (`accept`, `reject`, `complete`) require checking caller identity against `receiver_agent_id` or verifying admin/operator authorization.
  - **Audit Immutability**: All state-changing handoff actions (`initiate`, `accept`, `reject`, `complete`, `cancel`) must emit a cryptographically chained audit row into SQLite WAL.
  - **Payload Integrity**: SHA-256 signature verification prevents in-flight tampering or unauthorized parameter substitution.
  - **Denial of Service Prevention**: Request rate throttling and capacity bounds (`max_store_bytes`).

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Authorization Check | Fact | Method `can_agent_act(&self, agent_id: &str, action: &str)` in `HandoffRecord`. |
| Audit Row Invariant | Fact | Every consequential state transition writes exactly 1 audit row via `classify_and_emit` or `dispatch::recorded_call`. |
| Security Policy Tests | Fact | Asserted via Criterion `H7` in `tools/test_handoff_suites.py`. |
