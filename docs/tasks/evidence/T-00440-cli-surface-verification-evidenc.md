# T-00440 — Documentation Index Control / CLI surface: Verification & Evidence

## 1. Verification Overview
This task concludes the CLI Surface sub-epic (T-00431..T-00440) for Documentation Index Control in `code/aiosh-rust/aiosh-cli`.

## 2. Test Execution & Evidence

### A. Crate Unit Tests (`cargo test --manifest-path code/aiosh-rust/Cargo.toml`)
```text
running 16 tests
test doc_index::tests::test_doc_index_manifest_default_is_valid ... ok
test doc_index::tests::test_doc_index_manifest_duplicate_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_section_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_title_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_version_fails ... ok
test doc_index::tests::test_doc_index_manifest_malformed_json_fails ... ok
test doc_index::tests::test_doc_index_manifest_links_limit_fails ... ok
test doc_index::tests::test_doc_index_manifest_query_helpers ... ok
test doc_index::tests::test_doc_index_manifest_roundtrip_happy ... ok
test doc_index_service::tests::test_parse_markdown_links_happy ... ok
test doc_index_service::tests::test_build_doc_index_missing_file_error ... ok
test doc_index_service::tests::test_parse_markdown_title_happy ... ok
test doc_index_service::tests::test_real_repo_docs_index_and_validation ... ok
test doc_index_service::tests::test_validate_doc_links_escape_detected ... ok
test doc_index_service::tests::test_validate_doc_links_and_build_index ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.04s

running 1 test
test task_cli_tests::test_cmd_doc_show_check_and_search ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.19s
```

### B. CLI Smoke Test Suite (`python code/aiosh-cli/tests/test_doc_cli_smoke.py`)
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

### C. System Invariants & Smokes
```text
PASS: task docs criteria (C1..C6)
PASS: security policy criteria (S1..S5)
PASS: test_toolchain_cli_smoke.py
PASS: test_toolchain_mcp_smoke.py
PASS: ci_suites unit tests (W1..W7)
PASS: ci_service unit tests (X1..X7)
```

## 3. Summary
The CLI Surface sub-epic (T-00431..T-00440) is verified and closed.
