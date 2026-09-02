# T-00604 — Evidence & Audit Trail / recovery & validation: Implementation

## 1. Implementation Scope
This task implements `recover_default_evidence_config`, `reconstruct_evidence_manifest`, and `reconcile_evidence_manifest` in `code/aiosh-rust/aiosh-core/src/evidence_service.rs` to support automated recovery and validation of Evidence & Audit Trail manifests.

## 2. Implementation Deliverables
- **`code/aiosh-rust/aiosh-core/src/evidence_service.rs`**:
  - `recover_default_evidence_config`: Restores in-memory canonical `EvidenceConfig` defaults.
  - `scan_evidence_directory`: Discovers and indexes evidence markdown artifacts from disk.
  - `reconstruct_evidence_manifest`: Rebuilds full `TaskEvidenceManifest` from live disk artifacts.
  - `reconcile_evidence_manifest`: Runs verification and generates aggregate `EvidenceTelemetry`.
  - Unit tests `test_recover_default_evidence_config` and `test_reconstruct_and_reconcile_evidence_manifest`.

## 3. Test Verification
```text
running 10 tests
test evidence_service::tests::test_build_and_verify_evidence_manifest_happy ... ok
test evidence_service::tests::test_build_evidence_record_invalid_paths_error ... ok
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok
test evidence_service::tests::test_collect_evidence_telemetry ... ok
test evidence_service::tests::test_compute_file_sha256_happy_and_missing ... ok
test evidence_service::tests::test_compute_file_sha256_with_config_size_limit ... ok
test evidence_service::tests::test_format_evidence_summary ... ok
test evidence_service::tests::test_recover_default_evidence_config ... ok
test evidence_service::tests::test_reconstruct_and_reconcile_evidence_manifest ... ok
test evidence_service::tests::test_verify_evidence_manifest_mismatch_and_missing ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 146 filtered out; finished in 0.05s
```
