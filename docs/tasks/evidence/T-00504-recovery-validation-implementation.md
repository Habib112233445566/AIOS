# T-00504 — Documentation Index Control / recovery & validation: Implementation

## 1. Implementation Scope
This task implements `recover_default_doc_index_config`, `validate_doc_index_catalog`, and `reconcile_doc_index` in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` to provide programmatic recovery from corrupted configurations, standalone catalog link validation, and atomic multi-document reconciliation.

## 2. Implementation Details
- `recover_default_doc_index_config() -> DocIndexConfig`:
  - Returns the fallback compile-time configuration (`root_dirs: ["docs"]`, `enforce_strict_links: true`).
- `validate_doc_index_catalog(repo_root, manifest) -> Result<DocIndexTelemetry, String>`:
  - Validates in-tree links across all indexed documents and returns calculated `DocIndexTelemetry` on success, or an explicit error string detailing broken link counts.
- `reconcile_doc_index(repo_root, doc_paths) -> Result<(DocIndexManifest, DocLinkValidationReport, DocIndexTelemetry), String>`:
  - Idempotently loads documents, builds the manifest, verifies link targets, and generates telemetry summaries in a single pass.

## 3. Test Verification
```text
running 1 test
test doc_index_service::tests::test_validate_and_reconcile_doc_index_happy ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 128 filtered out; finished in 0.41s
```
