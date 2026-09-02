# T-00608 — Evidence & Audit Trail / recovery & validation: Hardening

## 1. Hardening Scope
This task verifies the defensive boundaries, memory limits, and fail-closed error handling in Evidence & Audit Trail recovery and validation routines.

## 2. Hardening Invariants & Defenses
- **16 MiB File Read Limit (`MAX_DOC_BYTES`)**: Files exceeding 16 MiB during checksum computation fail loudly with `Err("File ... exceeds max size cap ...")`.
- **64 KiB Configuration Cap**: Prevents configuration poisoning.
- **Fail-Closed Missing Directory Handling**: Missing evidence directories return explicit error messages instead of panicking or silently proceeding with empty results.
- **No Resource Leaks**: All directory reads use RAII and clean up open file descriptors immediately upon error or completion.

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
