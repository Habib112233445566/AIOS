# T-00594 — Evidence & Audit Trail / documentation: Implementation

## 1. Implementation Scope
This task implements `format_evidence_summary` in `code/aiosh-rust/aiosh-core/src/evidence_service.rs` to format human-readable console and log summaries of `TaskEvidenceManifest`.

## 2. Implementation Deliverables
- **`code/aiosh-rust/aiosh-core/src/evidence_service.rs`**:
  - `format_evidence_summary(&TaskEvidenceManifest) -> String` formatting task IDs, steps, relative paths, truncated hashes, and statuses.
  - Unit test `test_format_evidence_summary` validating populated and empty manifest renderings.

## 3. Test Verification
```text
running 1 test
test evidence_service::tests::test_format_evidence_summary ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 152 filtered out; finished in 0.00s
```
