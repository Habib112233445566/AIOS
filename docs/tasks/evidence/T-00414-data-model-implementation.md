# T-00414 — Documentation Index Control / data model: Implementation

## 1. Implementation Scope
This task implements the core data structures and methods for Documentation Index Control in `code/aiosh-rust/aiosh-core/src/doc_index.rs`.

## 2. Implementation Details
- **`DocIndexEntry`**: Struct storing repository-relative `path`, `title`, `section`, optional `task_range`, and outbound `links`.
- **`DocIndexManifest`**: Container storing manifest `version` and list of `DocIndexEntry` items.
- **Methods**:
  - `from_json(json_str: &str) -> Result<DocIndexManifest, String>`: Deserializes and validates the manifest.
  - `to_json(&self) -> Result<String, String>`: Validates and serializes the manifest to pretty-printed JSON.
  - `find_entry_by_path(&self, path: &str) -> Option<&DocIndexEntry>`: O(N) lookup by path.
  - `find_entries_by_section(&self, section: &str) -> Vec<&DocIndexEntry>`: Filters entries by section.
  - `validate(&self) -> Result<(), String>`: Enforces non-empty strings and path uniqueness.

## 3. Unit Test Verification
```text
running 5 tests
test doc_index::tests::test_doc_index_manifest_duplicate_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_version_fails ... ok
test doc_index::tests::test_doc_index_manifest_query_helpers ... ok
test doc_index::tests::test_doc_index_manifest_roundtrip_happy ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
```
