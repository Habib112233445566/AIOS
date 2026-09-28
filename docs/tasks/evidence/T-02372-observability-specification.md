# Task Evidence: T-02372 (Audit Chain Extensions / observability: Specification)

## 1. Metadata
- **Task ID:** `T-02372`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Observability Specification
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 8: Observability (2/10) — Specification

---

## 2. Technical Specification

### 2.1 Constants & Types
```rust
pub const AUDITOBS_ERR_VALIDATION: &str = "AUDITOBS_ERR_VALIDATION";
pub const AUDITOBS_ERR_QUERY: &str = "AUDITOBS_ERR_QUERY";
pub const AUDITOBS_ERR_IO: &str = "AUDITOBS_ERR_IO";
pub const MAX_TELEMETRY_TEXT_LEN: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditChainObservabilityReport {
    pub generated_at_utc: String,
    pub total_rows: usize,
    pub total_extended_rows: usize,
    pub total_causal_links: usize,
    pub total_signed_events: usize,
    pub unique_actors_count: usize,
    pub unique_tools_count: usize,
    pub unique_sessions_count: usize,
    pub unique_traces_count: usize,
    pub outcomes_by_type: HashMap<String, usize>,
    pub active_policy_mode: String,
    pub db_file_bytes: u64,
    pub chain_integrity_checked: bool,
    pub chain_integrity_ok: bool,
    pub is_healthy: bool,
}
```

### 2.2 Contract & Method Invariants

1. **`sanitize_telemetry_text(s: &str) -> String`**:
   - Removes ASCII/Unicode control characters (`!c.is_control()`).
   - Bounds length to $\le 256$ Unicode scalar values.
   - Trims leading and trailing whitespace.

2. **`AuditChainObservabilityReport::generate(service: &AuditChainService) -> Result<Self, String>`**:
   - Queries `audit_ring` SQLite table for:
     - `COUNT(*)` as `total_rows`.
     - `COUNT(provenance_json)` or extended fields as `total_extended_rows`.
     - `COUNT(DISTINCT actor)` as `unique_actors_count`.
     - `COUNT(DISTINCT tool)` as `unique_tools_count`.
     - Group-by counts for `outcome` (`success`, `denied`, `error`).
   - Aggregates causal links and signature counts.
   - Extracts active security policy mode (`service.policy().mode`).
   - Inspects SQLite database file size in bytes via filesystem metadata.
   - Computes `is_healthy`: true if database connection is readable and verified rows chain consistently.

3. **`validate(&self) -> Result<(), String>`**:
   - `generated_at_utc` must be non-empty valid ISO 8601 string.
   - `total_extended_rows <= total_rows`.
   - `unique_actors_count <= total_rows`, `unique_tools_count <= total_rows`.

---

## 3. Acceptance Confirmation
- [x] Input, output, error cases, and telemetry shapes defined.
- [x] Sanitization and bounds checking specified.
- [x] Standard error codes (`AUDITOBS_ERR_*`) documented.
