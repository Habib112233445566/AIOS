# T-00922 — Agent Handoff Protocol / Core Service: Specification

## 1. Rust Service Interface & Persistence

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffStore {
    pub records: HashMap<String, HandoffRecord>,
    pub signature_index: HashMap<String, String>,
}

impl HandoffStore {
    pub fn new() -> Self;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;

    pub fn initiate_handoff(
        &mut self,
        sender: &str,
        receiver: &str,
        task_id: Option<u32>,
        context_summary: &str,
        payload_json: &str,
        priority: HandoffPriority,
    ) -> HandoffRecord;

    pub fn accept_handoff(&mut self, id: &str, notes: Option<&str>) -> Result<HandoffRecord, String>;
    pub fn reject_handoff(&mut self, id: &str, notes: Option<&str>) -> Result<HandoffRecord, String>;
    pub fn complete_handoff(&mut self, id: &str, notes: Option<&str>) -> Result<HandoffRecord, String>;
    pub fn cancel_handoff(&mut self, id: &str, notes: Option<&str>) -> Result<HandoffRecord, String>;

    pub fn get_by_id(&self, id: &str) -> Option<&HandoffRecord>;
    pub fn list_active(&self) -> Vec<HandoffRecord>;
    pub fn list_all(&self) -> Vec<HandoffRecord>;
    pub fn to_report(&self) -> HandoffReport;

    pub fn save_to_path(&self, path: &Path) -> Result<(), String>;
    pub fn load_from_path(path: &Path) -> Result<Self, String>;
    pub fn load_or_recover(path: &Path) -> (Self, Option<String>);
}
```

## 2. Invariants & Error Cases
- State transitions enforce strict state validation:
  - `accept_handoff` requires `status == Pending`.
  - `reject_handoff` requires `status == Pending`.
  - `complete_handoff` requires `status == Accepted`.
  - `cancel_handoff` requires `status == Pending || status == Accepted`.
- Replay deduplication: If identical in-flight handoff exists, returns existing record.
