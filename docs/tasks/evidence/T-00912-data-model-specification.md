# T-00912 — Agent Handoff Protocol / Data Model: Specification

## 1. Rust Data Structures & Enums

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffStatus {
    Pending,
    Accepted,
    Rejected,
    Completed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffRecord {
    pub id: String,
    pub signature: String,
    pub sender_agent_id: String,
    pub receiver_agent_id: String,
    pub task_id: Option<u32>,
    pub context_summary: String,
    pub payload_json: String,
    pub priority: HandoffPriority,
    pub status: HandoffStatus,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub resolution_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffReport {
    pub timestamp_utc: String,
    pub total_handoffs: u32,
    pub active_handoffs: u32,
    pub completed_handoffs: u32,
    pub records: Vec<HandoffRecord>,
}
```

## 2. Invariants & Determinism
- `id` format: `HND-<signature[..8]>`.
- `signature`: SHA-256 hex string computed from `sender`, `receiver`, `task_id`, and `payload_json`.
- `validate_handoff_record`: Verifies non-empty fields, prefix matching, and RFC-3339 timestamps.
