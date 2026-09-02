# T-00822 — Regression Triage / core service: Specification

## 1. Service Specification

```rust
pub struct TriageStore {
    records: HashMap<String, TriageRecord>, // Keyed by hex SHA-256 signature
    id_index: HashMap<String, String>,      // Keyed by TRG-xxxxxxxx ID
}

impl TriageStore {
    pub fn new() -> Self;
    pub fn record_failure(
        &mut self,
        test_target: &str,
        suite_name: &str,
        error_message: &str,
        repro_cmd: &str,
        severity: TriageSeverity,
    ) -> TriageRecord;
    pub fn ingest_ci_summary(&mut self, summary: &RunSummary) -> usize;
    pub fn get_by_id(&self, id: &str) -> Option<&TriageRecord>;
    pub fn get_by_signature(&self, signature: &str) -> Option<&TriageRecord>;
    pub fn update_status(
        &mut self,
        id: &str,
        status: TriageStatus,
        notes: Option<String>,
    ) -> Result<&TriageRecord, String>;
    pub fn resolve(&mut self, id: &str, notes: &str) -> Result<&TriageRecord, String>;
    pub fn to_report(&self) -> TriageReport;
    pub fn save_to_path(&self, path: &Path) -> Result<(), String>;
    pub fn load_from_path(path: &Path) -> Result<Self, String>;
}
```

## 2. Invariants & Error Contracts
- Deduplication: If a failure with matching signature is already in the store, `occurrences` increments and `last_observed_at` updates to current timestamp.
- Size Bounds: File loading enforces a strict 1 MiB size cap.
- Atomic Save: Saving formats JSON deterministically and uses standard file write buffers.
