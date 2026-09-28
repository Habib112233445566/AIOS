# Task Evidence: T-02392 - Audit Chain Extensions: Recovery & Validation Specification

## Goal
Specify data contracts, diagnostics, and recovery algorithms for the Audit Chain Extensions recovery & validation subsystem (`audit_chain_recovery.rs`).

## 1. Domain Types and Interfaces

### 1.1 Issue Severity and Codes
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditChainIssueSeverity {
    Fatal,
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditChainIssueCode {
    HashDiscontinuity,
    InvalidJson,
    SignatureMismatch,
    DanglingCausalLink,
    CausalCycleDetected,
    TimeInversion,
}
```

### 1.2 Validation Report
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainValidationReport {
    pub total_events: usize,
    pub healthy_events: usize,
    pub issues: Vec<AuditChainValidationIssue>,
    pub is_valid: bool,
    pub can_auto_repair: bool,
}
```

### 1.3 Recovery Result
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainRecoveryResult {
    pub ok: bool,
    pub backup_path: Option<String>,
    pub actions: Vec<AuditChainRepairAction>,
    pub repaired_count: usize,
    pub post_validation: AuditChainValidationReport,
}
```

## 2. Invariants & Guarantees
- **Atomic Pre-flight Snapshotting**: Prior to applying any forward-repair event, an atomic copy of the SQLite database file is created at `<db_path>.backup.<timestamp>`.
- **Forward-Repair Only**: Historical rows in the audit ring are strictly immutable and never rewritten or deleted. Recovery resolves chain disruptions by appending a cryptographically sealed `repair` anchor event linking the last verifiable state forward.
- **Fail-Safe Envelope**: Invalidation reports detail the precise `row_id` and `event_hash` where each failure occurred.

## 3. Surface Contracts
- CLI:
  - `aiosh audit validate [--json]`
  - `aiosh audit repair [--backup-dir <path>] [--json]`
- MCP:
  - `aios.audit.validate`: Runs structural and cryptographic validation.
  - `aios.audit.repair`: Executes atomic backup and appends a forward repair anchor.
