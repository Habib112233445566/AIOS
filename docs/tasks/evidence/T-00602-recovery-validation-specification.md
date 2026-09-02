# T-00602 — Evidence & Audit Trail / recovery & validation: Specification

## 1. Specification Overview
This specification formalizes the recovery and validation interfaces for Evidence & Audit Trail in AIOS, covering default configuration restoration, full disk-based manifest reconstruction, and end-to-end reconciliation.

## 2. Interface Contracts

### A. Configuration Recovery (`recover_default_evidence_config`)
```rust
pub fn recover_default_evidence_config() -> EvidenceConfig {
    EvidenceConfig::default()
}
```
- **Inputs**: None.
- **Outputs**: Valid `EvidenceConfig` populated with canonical defaults:
  - `evidence_dir`: `"docs/tasks/evidence"`
  - `max_file_bytes`: `16777216` (16 MiB)
  - `allowed_extensions`: `[".md", ".json"]`
  - `enforce_checksum`: `true`

### B. Live Manifest Reconstruction (`reconstruct_evidence_manifest`)
```rust
pub fn reconstruct_evidence_manifest(
    repo_path: &Path,
    task_range: &str,
    epic_name: &str,
) -> Result<TaskEvidenceManifest, String>
```
- **Inputs**: `repo_path: &Path`, `task_range: &str`, `epic_name: &str`.
- **Outputs**: `Ok(TaskEvidenceManifest)` containing all discovered evidence artifacts from `docs/tasks/evidence/`.
- **Errors**: Emits `Err` if scanning directory fails or contains non-UTF8 paths.

### C. End-to-End Reconciliation (`reconcile_evidence_manifest`)
```rust
pub fn reconcile_evidence_manifest(
    repo_path: &Path,
    manifest: &TaskEvidenceManifest,
) -> Result<(EvidenceVerificationReport, EvidenceTelemetry), String>
```
- **Inputs**: `repo_path: &Path`, `manifest: &TaskEvidenceManifest`.
- **Outputs**: Tuple of `(EvidenceVerificationReport, EvidenceTelemetry)`.
- **Errors**: Emits `Err` if filesystem validation fails.

## 3. Reused vs. New Interfaces
- **Reused**:
  - `aiosh-core::evidence_service::{scan_evidence_files, verify_evidence_manifest, collect_evidence_telemetry}`.
  - `aiosh-core::evidence::{TaskEvidenceManifest, EvidenceRecord, EvidenceVerificationReport, EvidenceTelemetry}`.
  - `aiosh-core::evidence_config::EvidenceConfig`.
- **New (AIOS-Specific)**:
  - `recover_default_evidence_config`.
  - `reconstruct_evidence_manifest`.
  - `reconcile_evidence_manifest`.
