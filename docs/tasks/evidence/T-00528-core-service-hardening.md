# T-00528 — Evidence & Audit Trail / core service: Hardening

## 1. Hardening Overview
This task hardens the core evidence services (`compute_file_sha256`, `build_evidence_record`, `verify_evidence_manifest`) and CLI/MCP interfaces against resource exhaustion, malformed paths, unhandled I/O failures, and silent error drops.

## 2. Hardening Measures
1. **File Read Caps (`MAX_DOC_BYTES`)**:
   - `compute_file_sha256` checks file metadata length before reading and rejects any file exceeding 16 MiB with an explicit error.
2. **Deterministic Error Tracking**:
   - `verify_evidence_manifest` isolates I/O errors per record, appending structured descriptions to `missing_files` or `hash_mismatches` while continuing to verify remaining records.
3. **Repository Containment**:
   - Relative paths are strictly validated to prevent directory traversal (`..`) or absolute path escapes.
4. **Standardized Result Envelopes & Auditing**:
   - Both CLI (`aiosh evidence`) and MCP (`aios.evidence.*`) return structured JSON envelopes and emit SHA-256 hash-chained audit rows to SQLite WAL.

## 3. Test Verification
```text
running 5 tests
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok
test evidence_service::tests::test_build_evidence_record_invalid_paths_error ... ok
test evidence_service::tests::test_compute_file_sha256_happy_and_missing ... ok
test evidence_service::tests::test_build_and_verify_evidence_manifest_happy ... ok
test evidence_service::tests::test_verify_evidence_manifest_mismatch_and_missing ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 138 filtered out; finished in 0.03s
```
