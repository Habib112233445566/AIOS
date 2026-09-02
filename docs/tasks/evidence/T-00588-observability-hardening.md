# T-00588 — Evidence & Audit Trail / observability: Hardening

## 1. Hardening Scope
This task verifies and documents the defensive hardening controls applied to Evidence & Audit Trail observability, diagnostics, and telemetry calculations.

## 2. Hardening Measures
- **512-Byte Outcome Clamping**:
  - Structured audit trail logs clamp long diagnostic strings (`clamp_str(512)`) before appending to SQLite WAL, preventing storage bloat.
- **Manifest Record Bound**:
  - `TaskEvidenceManifest::validate()` rejects manifests containing $>10000$ records, guarding against memory exhaustion.
- **16 MiB Checksum Caps**:
  - `compute_file_sha256_with_config` strictly rejects files exceeding `max_file_bytes` (16 MiB), preventing out-of-memory errors.
- **Fail-Safe Telemetry Defaults**:
  - Empty or unparseable reports default to bounded zero-state metrics without panic or null pointer dereferences.

## 3. Test Verification
```text
running 23 tests
test evidence::tests::test_evidence_record_path_traversal ... ok
test evidence::tests::test_evidence_record_invalid_status ... ok
test evidence::tests::test_evidence_record_invalid_hash ... ok
test evidence::tests::test_evidence_record_task_id_bounds ... ok
test evidence::tests::test_evidence_record_valid ... ok
test evidence::tests::test_evidence_step_as_str_all_variants ... ok
test evidence::tests::test_task_evidence_manifest_duplicate_error ... ok
test evidence::tests::test_task_evidence_manifest_roundtrip_and_query ... ok
test evidence_service::tests::test_collect_evidence_telemetry ... ok
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok
test evidence_service::tests::test_compute_file_sha256_with_config_size_limit ... ok
test evidence_service::tests::test_compute_file_sha256_happy_and_missing ... ok
test evidence_service::tests::test_verify_evidence_manifest_mismatch_and_missing ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; finished in 0.02s
```
