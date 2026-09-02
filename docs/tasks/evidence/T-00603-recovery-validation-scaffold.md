# T-00603 — Evidence & Audit Trail / recovery & validation: Scaffold

## 1. Scaffold Scope
This task scaffolds recovery and validation functions (`scan_evidence_directory`, `recover_default_evidence_config`, `reconstruct_evidence_manifest`, and `reconcile_evidence_manifest`) in `code/aiosh-rust/aiosh-core/src/evidence_service.rs`.

## 2. Scaffold Signatures
```rust
pub fn scan_evidence_directory(
    repo_root: &Path,
    task_filter: Option<u32>,
) -> Result<Vec<EvidenceRecord>, String>;

pub fn recover_default_evidence_config() -> EvidenceConfig;

pub fn reconstruct_evidence_manifest(
    repo_path: &Path,
    task_range: &str,
    epic_name: &str,
) -> Result<TaskEvidenceManifest, String>;

pub fn reconcile_evidence_manifest(
    repo_path: &Path,
    manifest: &TaskEvidenceManifest,
) -> Result<(EvidenceVerificationReport, EvidenceTelemetry), String>;
```

## 3. Test Verification
Compiles cleanly across workspace crates.
