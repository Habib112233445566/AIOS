# T-00512 — Evidence & Audit Trail / data model: Specification

## 1. Specification Overview
This specification formalizes the data models, types, validation rules, and canonical serialization format for Evidence & Audit Trail in AIOS.

## 2. Data Structures

### A. Sub-Epic Step Identifier (`EvidenceStep`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceStep {
    Research,
    Spec,
    Scaffold,
    Implementation,
    UnitTest,
    Integration,
    SecurityReview,
    Hardening,
    Documentation,
    Verification,
}
```

### B. Single Evidence Record (`EvidenceRecord`)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub task_id: u32,
    pub step: EvidenceStep,
    pub file_path: String,
    pub sha256_hash: String,
    pub timestamp_utc: String,
    pub status: String,
    pub summary: Option<String>,
}
```

### C. Task Evidence Manifest (`TaskEvidenceManifest`)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskEvidenceManifest {
    pub epic_name: String,
    pub task_range: String,
    pub generated_at: String,
    pub records: Vec<EvidenceRecord>,
}
```

### D. Verification Report (`EvidenceVerificationReport`)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceVerificationReport {
    pub total_records: usize,
    pub valid_records: usize,
    pub missing_files: Vec<String>,
    pub hash_mismatches: Vec<String>,
    pub is_valid: bool,
}
```

## 3. Invariants & Validation Rules
1. **SHA-256 Format**: `sha256_hash` must be exactly 64 lowercase hexadecimal characters `[0-9a-f]{64}`.
2. **Path Confinement**: `file_path` must be non-empty, relative, and must not escape repository bounds (`..`).
3. **ISO-8601 Timestamp**: `timestamp_utc` and `generated_at` must be valid ISO-8601 UTC strings.
4. **Step Coverage**: A completed 10-step sub-epic must contain all 10 `EvidenceStep` variants.
5. **Canonical Serialization**: JSON serialization must follow deterministic canonical JSON rules (`canonical_json`).

## 4. Reused vs. New Interfaces
- **Reused**:
  - `aiosh-core::canonical::{canonical, sha256_hex, utcnow_iso}`.
  - `aiosh-core::audit::AuditRing`.
- **New (AIOS-Specific)**:
  - `EvidenceStep`, `EvidenceRecord`, `TaskEvidenceManifest`, `EvidenceVerificationReport`.
