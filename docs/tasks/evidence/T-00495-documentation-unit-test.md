# T-00495 — Documentation Index Control / documentation: Unit Test

## 1. Unit Test Scope
This task tests `format_doc_index_summary` across empty manifests, single entries, and multi-section documentation catalogs.

## 2. Test Cases & Coverage
1. `test_format_doc_index_summary_happy`:
   - Single document formatting: `[General] Overview (docs/README.md)`.
2. `test_format_doc_index_summary_empty`:
   - Empty manifest fallback: `(no documents indexed)`.
3. `test_format_doc_index_summary_multiple`:
   - Multi-entry formatting across distinct sections (`[General]` and `[Strategy]`).

## 3. Test Execution Output
```text
running 3 tests
test doc_index_service::tests::test_format_doc_index_summary_empty ... ok
test doc_index_service::tests::test_format_doc_index_summary_happy ... ok
test doc_index_service::tests::test_format_doc_index_summary_multiple ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 123 filtered out; finished in 0.00s
```
