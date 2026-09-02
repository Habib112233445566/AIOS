# T-00523 — Evidence & Audit Trail / core service: Scaffold

## 1. Scaffold Scope
This task creates `code/aiosh-rust/aiosh-core/src/evidence_service.rs` and registers `evidence_service` in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## 2. Scaffold Signatures
- `compute_file_sha256(path: &Path) -> Result<String, String>`
- `build_evidence_record(repo_root: &Path, rel_path: &str, task_id: u32, step: EvidenceStep, summary: Option<String>) -> Result<EvidenceRecord, String>`
- `verify_evidence_manifest(repo_root: &Path, manifest: &TaskEvidenceManifest) -> Result<EvidenceVerificationReport, String>`
- `check_evidence_policy(grant: Option<&str>, tool_name: &str) -> Result<(), String>`

## 3. Test Verification
```text
running 1 test
test evidence_service::tests::test_compute_file_sha256_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 138 filtered out; finished in 0.00s
```
