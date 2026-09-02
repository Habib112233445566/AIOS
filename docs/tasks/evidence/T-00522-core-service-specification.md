# T-00522 — Evidence & Audit Trail / core service: Specification

## 1. Specification Overview
This specification formalizes the core service interfaces, functions, inputs, outputs, error conditions, and security invariants for Evidence & Audit Trail in `aiosh-core`.

## 2. Core Function Signatures

### A. SHA-256 Checksum Computation (`compute_file_sha256`)
```rust
pub fn compute_file_sha256(path: &Path) -> Result<String, String>
```
- **Inputs**: `path: &Path` to target file.
- **Outputs**: 64-character lowercase hexadecimal SHA-256 string.
- **Errors**:
  - File not found.
  - File exceeds `MAX_DOC_BYTES` (16 MiB).
  - File I/O read errors.

### B. Evidence Record Constructor (`build_evidence_record`)
```rust
pub fn build_evidence_record(
    repo_root: &Path,
    rel_path: &str,
    task_id: u32,
    step: EvidenceStep,
    summary: Option<String>,
) -> Result<EvidenceRecord, String>
```
- **Inputs**:
  - `repo_root: &Path`: Base repository checkout directory.
  - `rel_path: &str`: Repository-relative path to evidence file.
  - `task_id: u32`: Task ID (1..10000).
  - `step: EvidenceStep`: Sub-epic lifecycle step.
  - `summary: Option<String>`: Optional human summary.
- **Outputs**: Validated `EvidenceRecord`.
- **Errors**:
  - Out-of-bounds relative path or absolute path.
  - File missing or unreadable on disk.
  - Task ID or field bounds violation.

### C. Manifest Verification (`verify_evidence_manifest`)
```rust
pub fn verify_evidence_manifest(
    repo_root: &Path,
    manifest: &TaskEvidenceManifest,
) -> Result<EvidenceVerificationReport, String>
```
- **Inputs**:
  - `repo_root: &Path`: Base repository directory.
  - `manifest: &TaskEvidenceManifest`: Manifest containing records to verify.
- **Outputs**: `EvidenceVerificationReport` detailing:
  - `total_records: usize`
  - `valid_records: usize`
  - `missing_files: Vec<String>`
  - `hash_mismatches: Vec<String>`
  - `is_valid: bool`
- **Errors**:
  - Manifest validation fails (e.g. duplicate entries or empty fields).

### D. Security Policy Enforcement (`check_evidence_policy`)
```rust
pub fn check_evidence_policy(grant: Option<&str>, tool_name: &str) -> Result<(), String>
```
- **Rules**:
  - Read-only operations (`aios.evidence.get`, `evidence.verify`) pass without grants.
  - Mutating operations (`aios.evidence.record`, `evidence.set`) require valid PEP grant token.

## 3. Reused vs. New Interfaces
- **Reused**:
  - `aiosh_core::canonical::{sha256_hex, utcnow_iso}`.
  - `aiosh_core::evidence::{EvidenceRecord, EvidenceStep, TaskEvidenceManifest, EvidenceVerificationReport}`.
- **New (AIOS-Specific)**:
  - `compute_file_sha256`, `build_evidence_record`, `verify_evidence_manifest`, `check_evidence_policy`.
