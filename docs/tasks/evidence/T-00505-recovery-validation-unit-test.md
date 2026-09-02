# T-00505 — Documentation Index Control / recovery & validation: Unit Test

## 1. Unit Test Scope
This task tests `recover_default_doc_index_config`, `validate_doc_index_catalog`, and `reconcile_doc_index` in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` covering happy paths, broken link detection errors, missing file failures, and configuration recovery.

## 2. Test Cases & Coverage
1. `test_recover_default_doc_index_config_happy`:
   - Verifies default parameters (`root_dirs: ["docs"]`, `enforce_strict_links: true`).
2. `test_validate_and_reconcile_doc_index_happy`:
   - Verifies end-to-end reconciliation across interrelated valid markdown documents.
3. `test_validate_doc_index_catalog_broken_link_error`:
   - Verifies that broken links produce an explicit `Err` string detailing the count of broken links.
4. `test_reconcile_doc_index_missing_file_error`:
   - Verifies that attempting to reconcile a non-existent document file fails with `"Document not found"`.

## 3. Test Execution Output
```text
running 1 test
test doc_index_service::tests::test_reconcile_doc_index_missing_file_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 129 filtered out; finished in 0.01s
```
