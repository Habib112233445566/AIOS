# T-00418 — Documentation Index Control / data model: Hardening

## 1. Hardening Scope
This task implements resource constraints, boundary checks, and memory protection mechanisms for the Documentation Index Control data model in `code/aiosh-rust/aiosh-core/src/doc_index.rs`.

## 2. Hardening Measures
1. **Entry Count Cap (`MAX_ENTRIES = 10,000`)**:
   - `DocIndexManifest::validate()` rejects manifests exceeding 10,000 entries to prevent memory exhaustion and DoS from oversized catalog payloads.
2. **Link Vector Cap (`MAX_LINKS_PER_ENTRY = 1,000`)**:
   - Each `DocIndexEntry` is constrained to at most 1,000 outbound links, preventing path explosion attacks.
3. **Explicit Diagnostic Error Messages**:
   - Validation failures report exact numerical counts and offending paths (e.g. `DocIndexManifest exceeds maximum allowed entries (limit 10000, got 10001)`).
4. **Zero-Leak Memory Safety**:
   - Built with pure Rust standard collections with guaranteed deterministic deallocation.

## 3. Test Verification
```text
running 10 tests
test doc_index::tests::test_doc_index_manifest_empty_section_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_default_is_valid ... ok
test doc_index::tests::test_doc_index_manifest_duplicate_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_title_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_version_fails ... ok
test doc_index::tests::test_doc_index_manifest_links_limit_fails ... ok
test doc_index::tests::test_doc_index_manifest_malformed_json_fails ... ok
test doc_index::tests::test_doc_index_manifest_query_helpers ... ok
test doc_index::tests::test_doc_index_manifest_roundtrip_happy ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
```
