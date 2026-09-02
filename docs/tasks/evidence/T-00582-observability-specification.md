# T-00582 — Evidence & Audit Trail / observability: Specification

## 1. Specification Overview
This specification formalizes the telemetry schemas, audit logging events, diagnostic reporters, and error observability for Evidence & Audit Trail in AIOS.

## 2. Telemetry and Event Schemas

### A. Telemetry Data Model (`EvidenceTelemetry`)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceTelemetry {
    pub total_records: usize,
    pub valid_records: usize,
    pub missing_files_count: usize,
    pub hash_mismatches_count: usize,
    pub is_healthy: bool,
}
```

### B. Telemetry Collection Helper
```rust
pub fn collect_evidence_telemetry(report: &EvidenceVerificationReport) -> EvidenceTelemetry {
    EvidenceTelemetry {
        total_records: report.total_records,
        valid_records: report.valid_records,
        missing_files_count: report.missing_files.len(),
        hash_mismatches_count: report.hash_mismatches.len(),
        is_healthy: report.is_valid,
    }
}
```

### C. Audit Ring Event Schema (Happy Path)
- **Tool**: `evidence.hash` | `evidence.verify` | `evidence.scan` | `aios.evidence.hash` | `aios.evidence.verify` | `aios.evidence.scan`.
- **Outcome**: `"ok"` / `"success"`.
- **Outcome Detail**: 512-byte clamped summary:
  - `evidence.hash`: `"Computed SHA-256 for <path>: <hash_prefix>..."`.
  - `evidence.verify`: `"Verified <N> records: <V> valid, <M> missing, <H> mismatches, status=healthy"`.
  - `evidence.scan`: `"Discovered <N> evidence markdown artifacts under <dir>"`.

### D. Audit Ring Event Schema (Failure Path)
- **Tool**: `evidence.*` | `aios.evidence.*`.
- **Outcome**: `"error"` | `"refused"`.
- **Outcome Detail**:
  - Missing file / bad path: `"File not found: <path>"`.
  - Checksum mismatch / verification failure: `"Verification failed: <M> missing, <H> mismatches"`.
  - Oversized file cap: `"File <path> exceeds max size cap of 16777216 bytes"`.
  - Policy refusal: `"PermissionDenied: mutating evidence actions require a valid PEP grant"`.

## 3. Reused vs. New Interfaces
- **Reused**:
  - `aiosh-core::audit::AuditRing` and `aiosh-cli::emit` for WAL persistence.
  - `aiosh-core::evidence::{EvidenceRecord, EvidenceVerificationReport, TaskEvidenceManifest}`.
- **New (AIOS-Specific)**:
  - `EvidenceTelemetry` struct.
  - `collect_evidence_telemetry(&EvidenceVerificationReport) -> EvidenceTelemetry`.
