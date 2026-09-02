# T-00524 — Evidence & Audit Trail / core service: Implementation

## 1. Implementation Scope
This task implements the core service functions for Evidence & Audit Trail in `code/aiosh-rust/aiosh-core/src/evidence_service.rs` and configures PEP policy enforcement in `code/aiosh-rust/aiosh-core/src/pep.rs`.

## 2. Implemented Service Operations
- `compute_file_sha256(path: &Path) -> Result<String, String>`:
  - Validates file existence and 16 MiB size cap (`MAX_DOC_BYTES`).
  - Computes deterministic lowercase hex SHA-256 string.
- `build_evidence_record(repo_root: &Path, rel_path: &str, task_id: u32, step: EvidenceStep, summary: Option<String>) -> Result<EvidenceRecord, String>`:
  - Sanitizes relative path, computes disk SHA-256, and instantiates validated `EvidenceRecord`.
- `verify_evidence_manifest(repo_root: &Path, manifest: &TaskEvidenceManifest) -> Result<EvidenceVerificationReport, String>`:
  - Validates manifest records against disk files, compiling `missing_files`, `hash_mismatches`, `valid_records`, and `is_valid`.
- `check_evidence_policy(grant: Option<&str>, tool_name: &str) -> Result<(), String>`:
  - Enforces read-only vs. mutating PEP permission boundaries.
- `pep.rs`: Added `aios.evidence.record`, `evidence.record`, `aios.evidence.set`, and `evidence.set` to `is_irreversible`.

## 3. Test Verification
```text
running 4 tests
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok
test evidence_service::tests::test_compute_file_sha256_happy_and_missing ... ok
test evidence_service::tests::test_build_and_verify_evidence_manifest_happy ... ok
test evidence_service::tests::test_verify_evidence_manifest_mismatch_and_missing ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 138 filtered out; finished in 0.18s
```
