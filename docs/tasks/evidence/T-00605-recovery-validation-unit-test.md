# T-00605 — Evidence & Audit Trail / recovery & validation: Unit Test

## 1. Unit Test Scope
This task implements and executes unit tests for `recover_default_evidence_config`, `reconstruct_evidence_manifest`, `scan_evidence_directory`, and `reconcile_evidence_manifest` in `code/aiosh-rust/aiosh-core/src/evidence_service.rs`.

## 2. Test Cases & Coverage
1. **Configuration Recovery**:
   - Asserts in-memory defaults restore `evidence_dir = "docs/tasks/evidence"`, `max_file_bytes = 16 MiB`, and `enforce_checksum = true`.
2. **Reconstruction & Reconciliation**:
   - Asserts `reconstruct_evidence_manifest` scans on-disk files and populates matching `TaskEvidenceManifest` records with SHA-256 hashes.
   - Asserts `reconcile_evidence_manifest` generates valid reports and healthy telemetry.
3. **Filtered Scans**:
   - Asserts task filtering isolates target task IDs accurately.
4. **Degraded Manifest Detection**:
   - Asserts tampering with evidence files triggers `report.is_valid = false` and `telemetry.is_healthy = false`.
5. **Non-Existent Directory Error**:
   - Asserts scanning non-existent directories returns explicit `Err("Evidence directory not found: ...")`.

## 3. Test Verification Output
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
