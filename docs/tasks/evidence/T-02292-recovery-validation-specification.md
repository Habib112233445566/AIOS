# T-02292: Grant Lifecycle Recovery & Validation Specification

## 1. Domain Types & Enums

### 1.1 Issue Severity
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepGrantIssueSeverity {
    Fatal,   // Store unparseable or completely corrupted
    Error,   // Invariant violation (broken hierarchy, cycle, attenuation violation)
    Warning, // Expired grant still marked Active, unindexed field
}
```

### 1.2 Issue Codes
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PepGrantIssueCode {
    InvalidJson,
    SchemaViolation,
    CycleDetected,
    OrphanGrant,
    DepthInconsistency,
    AttenuationViolation,
    CascadeDesync,
    TimeInversion,
    ExpiredActive,
}
```

### 1.3 Validation Report
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantValidationIssue {
    pub grant_id: Option<String>,
    pub code: PepGrantIssueCode,
    pub severity: PepGrantIssueSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantValidationReport {
    pub total_grants: usize,
    pub healthy_grants: usize,
    pub issues: Vec<PepGrantValidationIssue>,
    pub is_valid: bool,
    pub can_auto_repair: bool,
}
```

### 1.4 Recovery & Repair Result
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantRepairAction {
    pub grant_id: String,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantRecoveryResult {
    pub ok: bool,
    pub backup_path: Option<String>,
    pub quarantine_path: Option<String>,
    pub actions: Vec<PepGrantRepairAction>,
    pub repaired_count: usize,
    pub post_validation: PepGrantValidationReport,
}
```

## 2. Validation Engine Rules

1. **Cycle Detection**:
   Traverse parent chain for each grant. If any grant appears twice in its ancestor chain, emit `CycleDetected` with `Error` severity.
2. **Orphan Grants**:
   If `parent_grant_id` is `Some(pid)` but `pid` does not exist in store, emit `OrphanGrant` with `Error` severity.
3. **Depth Consistency**:
   If `grant.depth != parent.depth + 1`, emit `DepthInconsistency` (`Error`).
   If `grant.depth > grant.max_depth` or `grant.max_depth > parent.max_depth`, emit `DepthInconsistency` (`Error`).
4. **Attenuation Enforcement**:
   Child scopes must be a subset of parent scopes: if any scope is not present in parent, emit `AttenuationViolation` (`Error`).
   Child allowed tools must be a subset of parent tools: if any tool is not present in parent, emit `AttenuationViolation` (`Error`).
5. **Cascade Desynchronization**:
   If parent status is `Revoked`, child status must also be `Revoked`. If child is `Active` or `Suspended`, emit `CascadeDesync` (`Error`).
6. **Temporal Consistency**:
   `issued_at <= expires_at`. If `expires_at < issued_at`, emit `TimeInversion` (`Error`).
   If `now > expires_at` and status is `Active`, emit `ExpiredActive` (`Warning`).

## 3. Recovery Engine Specification

1. **Atomic Backup**:
   Before modifying any file, write full snapshot to `<store_path>.bak.<timestamp>`.
2. **Quarantine of Unparseable Data**:
   If raw store contains unparseable lines or corrupt fragments, write unparseable payload to `<store_path>.quarantine.<timestamp>.json` and rebuild valid store.
3. **Auto-Repair Operations**:
   - **Orphans**: Update status to `Revoked` with note `"auto-recovery: orphan"`.
   - **Cascade Desync**: Update status to `Revoked` with note `"auto-recovery: cascade sync"`.
   - **Expired Active**: Update status to `Expired`.
4. **Atomic Swap**:
   Write repaired store to `<store_path>.tmp.<pid>` and perform atomic replace over `<store_path>`.
5. **Post-Validation**:
   Re-run validation engine and verify that `is_valid == true`.
