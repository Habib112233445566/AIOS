# T-00425 — Documentation Index Control / core service: Unit Test

## 1. Unit Test Scope
This task tests the Documentation Index Control core service (`doc_index_service.rs`) across happy, negative, and boundary test scenarios.

## 2. Test Cases & Coverage
1. `test_parse_markdown_title_happy`: Asserts accurate extraction of `# H1` headings and returns `None` for subheadings (`##`).
2. `test_parse_markdown_links_happy`: Verifies extraction of relative in-tree links (`[text](target.md)`), stripping anchor fragments, and ignoring external schemas (`http://`, `https://`, `mailto:`).
3. `test_validate_doc_links_and_build_index`: Verifies index generation from files and link validation flagging missing targets.
4. `test_validate_doc_links_escape_detected`: Tests that links attempting to escape the repository root (e.g. `../../../etc/passwd`) are detected and flagged in `broken_links`.
5. `test_build_doc_index_missing_file_error`: Tests that attempting to index a non-existent document path returns an explicit error envelope.

## 3. Test Execution Output
```text
running 5 tests
test doc_index_service::tests::test_parse_markdown_title_happy ... ok
test doc_index_service::tests::test_parse_markdown_links_happy ... ok
test doc_index_service::tests::test_build_doc_index_missing_file_error ... ok
test doc_index_service::tests::test_validate_doc_links_and_build_index ... ok
test doc_index_service::tests::test_validate_doc_links_escape_detected ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 108 filtered out; finished in 0.03s
```
