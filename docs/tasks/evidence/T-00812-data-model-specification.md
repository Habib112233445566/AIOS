# T-00812 — Regression Triage / data model: Specification

## 1. Data Model Specification

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriageStatus {
    Untriaged,
    Triaged,
    FixPending,
    Resolved,
    WontFix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriageSeverity {
    Blocker,
    Critical,
    Major,
    Minor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriageRecord {
    pub id: String,
    pub signature: String,
    pub test_target: String,
    pub suite_name: String,
    pub error_message: String,
    pub repro_command: String,
    pub first_observed_at: String,
    pub last_observed_at: String,
    pub occurrences: u32,
    pub status: TriageStatus,
    pub severity: TriageSeverity,
    pub blame_task_id: Option<u32>,
    pub blame_commit: Option<String>,
    pub resolution_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriageReport {
    pub timestamp_utc: String,
    pub total_records: u32,
    pub open_records: u32,
    pub resolved_records: u32,
    pub records: Vec<TriageRecord>,
}
```

## 2. Invariants & Helper Contracts
- `compute_failure_signature(test_target: &str, error_message: &str) -> String`: Computes hex-encoded SHA-256 hash over normalized `test_target` and sanitized `error_message`.
- `validate_triage_report(report: &TriageReport) -> Result<(), String>`: Enforces invariant consistency across counters and record array length.
