# T-00502 — Documentation Index Control / recovery & validation: Specification

## 1. Specification Overview
This specification formalizes the recovery and validation interfaces for Documentation Index Control in AIOS, covering default configuration restoration, full catalog validation, and end-to-end reconciliation.

## 2. Interface Contracts

### A. Configuration Recovery (`recover_default_doc_index_config`)
```rust
pub fn recover_default_doc_index_config() -> DocIndexConfig {
    DocIndexConfig::default()
}
```
- **Inputs**: None.
- **Outputs**: Valid `DocIndexConfig` populated with canonical defaults:
  - `root_dirs: ["docs"]`
  - `include_extensions: [".md", ".markdown"]`
  - `exclude_patterns: ["node_modules", "target", ".git"]`
  - `enforce_strict_links: true`

### B. Catalog Validation (`validate_doc_index_catalog`)
```rust
pub fn validate_doc_index_catalog(
    repo_root: &Path,
    manifest: &DocIndexManifest,
) -> Result<DocIndexTelemetry, String>
```
- **Inputs**: `repo_root: &Path`, `manifest: &DocIndexManifest`.
- **Outputs**: `Ok(DocIndexTelemetry)` if all links pass; `Err(String)` describing the broken link count if link failures or repo escapes occur.

### C. End-to-End Reconciliation (`reconcile_doc_index`)
```rust
pub fn reconcile_doc_index(
    repo_root: &Path,
    doc_paths: &[&str],
) -> Result<(DocIndexManifest, DocLinkValidationReport, DocIndexTelemetry), String>
```
- **Inputs**: `repo_root: &Path`, `doc_paths: &[&str]`.
- **Outputs**: Tuple containing the parsed `DocIndexManifest`, `DocLinkValidationReport`, and computed `DocIndexTelemetry`.
- **Errors**: Emits `Err` if any input document file is missing or unreadable.

## 3. Reused vs. New Interfaces
- **Reused**:
  - `aiosh-core::doc_index_service::{build_doc_index_from_paths, validate_doc_links, collect_doc_index_telemetry}`.
  - `aiosh-core::doc_index_config::DocIndexConfig`.
- **New (AIOS-Specific)**:
  - `recover_default_doc_index_config`.
  - `validate_doc_index_catalog`.
  - `reconcile_doc_index`.
