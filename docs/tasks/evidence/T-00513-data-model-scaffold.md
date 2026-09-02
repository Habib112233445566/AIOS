# T-00513 — Evidence & Audit Trail / data model: Scaffold

## 1. Scaffold Scope
This task creates the `code/aiosh-rust/aiosh-core/src/evidence.rs` module and registers it in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## 2. Scaffold Implementation
- `EvidenceStep` enum (representing 10 sub-epic steps).
- `EvidenceRecord` struct (task ID, step, file path, SHA-256 hash, UTC timestamp, status, summary).
- `TaskEvidenceManifest` struct (epic name, task range, generation timestamp, records list).
- `EvidenceVerificationReport` struct (counts, missing files, hash mismatches, boolean validity).
- Scaffold signatures for `validate()`, `from_json()`, and `to_json()`.

## 3. Test Verification
```text
running 1 test
test evidence::tests::test_task_evidence_manifest_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 130 filtered out; finished in 0.01s
```
