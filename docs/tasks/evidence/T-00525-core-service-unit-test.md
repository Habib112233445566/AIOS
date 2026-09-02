# T-00525 — Evidence & Audit Trail / core service: Unit Test

## 1. Unit Test Scope
This task tests the core service operations for Evidence & Audit Trail in `code/aiosh-rust/aiosh-core/src/evidence_service.rs` covering hash calculation, evidence record construction, path traversal defense, manifest verification (happy path, missing files, tampered checksums), and PEP policy enforcement.

## 2. Test Cases & Coverage
1. `test_compute_file_sha256_happy_and_missing`:
   - Computes SHA-256 for a real file; confirms missing file returns an explicit `Err`.
2. `test_build_and_verify_evidence_manifest_happy`:
   - Builds evidence records from files on disk and asserts `verify_evidence_manifest` passes with `is_valid: true`.
3. `test_verify_evidence_manifest_mismatch_and_missing`:
   - Modifies file contents to trigger hash mismatches and includes non-existent file paths to verify failure reporting in `missing_files` and `hash_mismatches`.
4. `test_check_evidence_policy_enforcement`:
   - Validates unauthenticated read-only operations (`aios.evidence.get`, `evidence.verify`) and PEP grant gating on mutating operations (`aios.evidence.record`).
5. `test_build_evidence_record_invalid_paths_error`:
   - Asserts empty paths, relative traversals (`../`), and absolute paths (`/etc/shadow`) are rejected with explicit errors.

## 3. Test Execution Output
```text
running 5 tests
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok
test evidence_service::tests::test_build_evidence_record_invalid_paths_error ... ok
test evidence_service::tests::test_compute_file_sha256_happy_and_missing ... ok
test evidence_service::tests::test_build_and_verify_evidence_manifest_happy ... ok
test evidence_service::tests::test_verify_evidence_manifest_mismatch_and_missing ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 138 filtered out; finished in 0.03s
```
