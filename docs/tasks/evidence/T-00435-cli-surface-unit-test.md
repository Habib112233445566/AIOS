# T-00435 — Documentation Index Control / CLI surface: Unit Test

## 1. Unit Test Scope
This task tests the CLI surface of Documentation Index Control (`aiosh doc`) using both in-crate tests and end-to-end Python smoke tests (`test_doc_cli_smoke.py`).

## 2. Test Cases & Coverage
1. `test_doc_show_prose`: Confirms `aiosh doc show` outputs human-readable catalog table.
2. `test_doc_show_json`: Confirms `aiosh doc show --json` outputs formatted `DocIndexManifest` with exit code 0.
3. `test_doc_check_prose`: Confirms `aiosh doc check` validates in-tree links and returns exit code 0.
4. `test_doc_check_json`: Confirms `aiosh doc check --json` serializes `DocLinkValidationReport` with `is_valid: true`.
5. `test_doc_search` / `test_doc_search_json`: Asserts query filtering for document paths and sections.
6. `test_doc_invalid_subcommand`: Negative test asserting exit code 2 and usage text on unknown subcommands.
7. `test_doc_search_missing_query`: Negative test asserting exit code 2 when search query is omitted.
8. `test_doc_check_broken_links_negative`: Negative test asserting exit code 1 and `is_valid: false` when pointing `--repo` at an isolated workspace with broken markdown links.

## 3. Test Execution Output
```text
PASS: aiosh doc show prose
PASS: aiosh doc show --json
PASS: aiosh doc check prose
PASS: aiosh doc check --json
PASS: aiosh doc search
PASS: aiosh doc search --json
PASS: aiosh doc invalid subcommand
PASS: aiosh doc search missing query
PASS: aiosh doc check broken link detection negative test
PASS: test_doc_cli_smoke.py
```
