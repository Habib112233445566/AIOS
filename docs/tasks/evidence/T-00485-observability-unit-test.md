# T-00485 — Documentation Index Control / observability: Unit Test

## 1. Unit Test Scope
This task tests `collect_doc_index_telemetry` across clean validation reports, reports containing broken links, and fallback behavior when no validation report is supplied.

## 2. Test Cases & Coverage
1. `test_collect_doc_index_telemetry_happy`:
   - Validates correct aggregation of `total_docs_indexed`, `total_links_checked`, zero broken links, and `is_healthy = true`.
2. `test_collect_doc_index_telemetry_with_broken_links`:
   - Validates detection of broken links, non-zero `broken_links_count`, and `is_healthy = false`.
3. `test_collect_doc_index_telemetry_none_report`:
   - Validates that when validation report is `None`, telemetry calculates total outbound links from the manifest entries directly and defaults to `is_healthy = true`.

## 3. Test Execution Output
```text
running 3 tests
test doc_index_service::tests::test_collect_doc_index_telemetry_happy ... ok
test doc_index_service::tests::test_collect_doc_index_telemetry_with_broken_links ... ok
test doc_index_service::tests::test_collect_doc_index_telemetry_none_report ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 120 filtered out; finished in 0.00s
```
