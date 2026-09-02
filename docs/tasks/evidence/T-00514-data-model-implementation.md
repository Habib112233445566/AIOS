# T-00514 — Evidence & Audit Trail / data model: Implementation

## 1. Implementation Scope
This task implements the core Evidence & Audit Trail data models in `code/aiosh-rust/aiosh-core/src/evidence.rs`.

## 2. Implementation Details
- `EvidenceStep`: Enum defining 10 sub-epic lifecycle steps (`Research`, `Spec`, `Scaffold`, `Implementation`, `UnitTest`, `Integration`, `SecurityReview`, `Hardening`, `Documentation`, `Verification`).
- `EvidenceRecord`: Models single task artifacts with `validate()` enforcing task ID bounds, relative non-escaping file paths, 64-char lowercase SHA-256 hex checksums, and valid status strings.
- `TaskEvidenceManifest`: Groups records with `from_json()`, `to_json()`, duplicate detection, `get_record()`, and `filter_by_step()` querying.
- `EvidenceVerificationReport`: Summary structure for verification results.

## 3. Test Verification
```text
running 4 tests
test evidence::tests::test_evidence_record_path_traversal ... ok
test evidence::tests::test_evidence_record_valid ... ok
test evidence::tests::test_evidence_record_invalid_hash ... ok
test evidence::tests::test_task_evidence_manifest_roundtrip_and_query ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 130 filtered out; finished in 0.00s
```
