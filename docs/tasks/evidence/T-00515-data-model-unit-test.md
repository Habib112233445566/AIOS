# T-00515 — Evidence & Audit Trail / data model: Unit Test

## 1. Unit Test Scope
This task tests the Evidence & Audit Trail data models in `code/aiosh-rust/aiosh-core/src/evidence.rs` across valid serialization, bounds validation, SHA-256 formatting, path security, duplicate detection, and query helpers.

## 2. Test Cases & Coverage
1. `test_evidence_record_valid`:
   - Valid record passes all validation constraints.
2. `test_evidence_record_invalid_hash`:
   - Rejects non-64-character, non-hexadecimal, and uppercase checksums.
3. `test_evidence_record_path_traversal`:
   - Rejects paths containing `..` or absolute path prefixes.
4. `test_evidence_record_task_id_bounds`:
   - Rejects `task_id == 0` and `task_id > 10000`; accepts `task_id == 1` and `task_id == 10000`.
5. `test_evidence_record_invalid_status`:
   - Rejects unknown status strings; allows only `"pass"`, `"fail"`, or `"pending"`.
6. `test_task_evidence_manifest_duplicate_error`:
   - Rejects duplicate `(task_id, step)` records in manifest.
7. `test_evidence_step_as_str_all_variants`:
   - Asserts string mappings for all 10 `EvidenceStep` variants.
8. `test_task_evidence_manifest_roundtrip_and_query`:
   - Tests JSON serialization/deserialization roundtrip, `get_record()`, and `filter_by_step()`.

## 3. Test Execution Output
```text
running 8 tests
test evidence::tests::test_evidence_record_invalid_status ... ok
test evidence::tests::test_evidence_record_path_traversal ... ok
test evidence::tests::test_evidence_record_task_id_bounds ... ok
test evidence::tests::test_evidence_record_invalid_hash ... ok
test evidence::tests::test_evidence_record_valid ... ok
test evidence::tests::test_evidence_step_as_str_all_variants ... ok
test evidence::tests::test_task_evidence_manifest_duplicate_error ... ok
test evidence::tests::test_task_evidence_manifest_roundtrip_and_query ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 130 filtered out; finished in 0.01s
```
