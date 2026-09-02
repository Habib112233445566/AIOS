# T-00415 — Documentation Index Control / data model: Unit Test

## 1. Unit Test Scope
This task tests the Documentation Index Control data model (`DocIndexEntry`, `DocIndexManifest`) across positive, negative, and boundary scenarios in `code/aiosh-rust/aiosh-core/src/doc_index.rs`.

## 2. Test Cases & Coverage
1. `test_doc_index_manifest_roundtrip_happy`: Validates serialization and deserialization symmetry with complex entries.
2. `test_doc_index_manifest_empty_version_fails`: Rejects manifests with empty version strings.
3. `test_doc_index_manifest_empty_path_fails`: Rejects entries with empty repository paths.
4. `test_doc_index_manifest_empty_title_fails`: Rejects entries with whitespace-only titles.
5. `test_doc_index_manifest_empty_section_fails`: Rejects entries with empty section groupings.
6. `test_doc_index_manifest_duplicate_path_fails`: Rejects manifests containing conflicting duplicate paths.
7. `test_doc_index_manifest_malformed_json_fails`: Rejects invalid JSON inputs with clear error envelopes.
8. `test_doc_index_manifest_query_helpers`: Verifies `find_entry_by_path` and `find_entries_by_section` lookup methods.
9. `test_doc_index_manifest_default_is_valid`: Asserts that `DocIndexManifest::default()` satisfies all schema invariants.

## 3. Test Execution Output
```text
running 9 tests
test doc_index::tests::test_doc_index_manifest_empty_section_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_duplicate_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_default_is_valid ... ok
test doc_index::tests::test_doc_index_manifest_empty_title_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_version_fails ... ok
test doc_index::tests::test_doc_index_manifest_malformed_json_fails ... ok
test doc_index::tests::test_doc_index_manifest_query_helpers ... ok
test doc_index::tests::test_doc_index_manifest_roundtrip_happy ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
```
