# T-00518 — Evidence & Audit Trail / data model: Hardening

## 1. Hardening Scope
This task hardens the data model validation logic in `code/aiosh-rust/aiosh-core/src/evidence.rs` with strict string length bounds, maximum collection caps, and exhaustive error descriptions.

## 2. Hardening Measures
1. **String Length Boundaries**:
   - `file_path`: <= 1024 characters.
   - `summary`: <= 4096 characters.
   - `epic_name`: <= 256 characters.
   - `task_range`: <= 64 characters.
2. **Collection Capacity Caps**:
   - `TaskEvidenceManifest::records`: capped at 10,000 items maximum to prevent memory exhaustion attacks.
3. **Explicit Error Messages**:
   - Every validation check returns structured error descriptions detailing violated fields.

## 3. Test Verification
```text
running 8 tests
test evidence::tests::test_evidence_record_invalid_status ... ok
test evidence::tests::test_evidence_record_invalid_hash ... ok
test evidence::tests::test_evidence_record_path_traversal ... ok
test evidence::tests::test_evidence_record_task_id_bounds ... ok
test evidence::tests::test_evidence_record_valid ... ok
test evidence::tests::test_evidence_step_as_str_all_variants ... ok
test evidence::tests::test_task_evidence_manifest_duplicate_error ... ok
test evidence::tests::test_task_evidence_manifest_roundtrip_and_query ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 130 filtered out; finished in 0.01s
```
