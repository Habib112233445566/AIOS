# T-00977 — Agent Handoff Protocol / Security Policy: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-1: Receiver Impersonation Attack
- **Threat**: Malicious subagent attempts to `accept` or `complete` a handoff intended for a different agent.
- **Mitigation**: `can_agent_act` strictly compares `actor_id` against `receiver_agent_id`, rejecting any unauthorized actor with an explicit `PermissionDenied` error.

### AS-2: Sender Cancellation Hijack
- **Threat**: Competing agent attempts to `cancel` an active or pending task sent by another agent.
- **Mitigation**: `can_agent_act` enforces `actor_id == sender_agent_id` for cancellation actions, preserving workflow integrity.

### AS-3: Privilege Escalation via Fake Role Names
- **Threat**: Agent passes crafted role names (e.g. `operator_fake`) to gain administrative bypass.
- **Mitigation**: Exact string matching on `"operator"`, `"admin"`, `"root"` after case normalization and whitespace trimming.
