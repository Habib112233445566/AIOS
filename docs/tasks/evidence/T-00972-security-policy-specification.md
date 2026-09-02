# T-00972 — Agent Handoff Protocol / Security Policy: Specification

## 1. Role-Based Action Matrix

| Role | Actions Permitted | Restrictions |
|---|---|---|
| Sender Agent | `initiate`, `cancel` | Can only cancel records where `sender_agent_id == caller_id`. |
| Receiver Agent | `accept`, `reject`, `complete` | Can only act on records where `receiver_agent_id == caller_id`. |
| Operator / Admin | `*` (all actions) | Full administrative oversight for emergency intervention. |
| Unrelated Agent | None | Rejections returned with `PermissionDenied`. |

## 2. Function Contract
```rust
impl HandoffRecord {
    pub fn can_agent_act(&self, actor_id: &str, action: &str) -> bool;
}
```
- Actions:
  - `"accept"` / `"reject"` / `"complete"`: requires `actor_id == self.receiver_agent_id || actor_id == "operator" || actor_id == "admin"`.
  - `"cancel"`: requires `actor_id == self.sender_agent_id || actor_id == "operator" || actor_id == "admin"`.
