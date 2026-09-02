# T-00460 — Documentation Index Control / configuration: Verification & Evidence

## 1. Verification Overview
This task concludes the Configuration sub-epic (T-00451..T-00460) for Documentation Index Control in `aiosh-core`, `aiosh-cli`, and `aiosh-mcp`.

## 2. Test Execution & Evidence

### A. Unit Tests (`cargo test --manifest-path code/aiosh-rust/Cargo.toml`)
```text
running 21 tests
test doc_index::tests::test_doc_index_manifest_default_is_valid ... ok
test doc_index::tests::test_doc_index_manifest_empty_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_section_fails ... ok
test doc_index::tests::test_doc_index_manifest_duplicate_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_title_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_version_fails ... ok
test doc_index::tests::test_doc_index_manifest_malformed_json_fails ... ok
test doc_index::tests::test_doc_index_manifest_links_limit_fails ... ok
test doc_index::tests::test_doc_index_manifest_query_helpers ... ok
test doc_index::tests::test_doc_index_manifest_roundtrip_happy ... ok
test doc_index_config::tests::test_doc_index_config_default_is_valid ... ok
test doc_index_config::tests::test_doc_index_config_from_env_fallback ... ok
test doc_index_config::tests::test_doc_index_config_roundtrip_happy ... ok
test doc_index_config::tests::test_doc_index_config_validation_failures ... ok
test doc_index_service::tests::test_parse_markdown_links_happy ... ok
test doc_index_service::tests::test_build_doc_index_missing_file_error ... ok
test doc_index_service::tests::test_parse_markdown_title_happy ... ok
test doc_index_service::tests::test_real_repo_docs_index_and_validation ... ok
test doc_index_config::tests::test_doc_index_config_from_path_and_missing ... ok
test doc_index_service::tests::test_validate_doc_links_escape_detected ... ok
test doc_index_service::tests::test_validate_doc_links_and_build_index ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.04s

running 1 test
test task_cli_tests::test_cmd_doc_show_check_and_search ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.14s

running 2 tests
test tests::test_toolchain_tools_in_manifest ... ok
test tests::test_mcp_doc_tools_execution ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

### B. CLI & MCP Smoke Test Suites
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
PASS: aiosh doc custom config valid
PASS: aiosh doc custom config missing negative test
PASS: test_doc_cli_smoke.py

PASS: aios.doc tools present in tools/list
PASS: aios.doc.index.get
PASS: aios.doc.check
PASS: aios.doc.search
PASS: aios.doc.search missing query negative test
PASS: test_doc_mcp_smoke.py
```

### C. System Invariants & CI Gates
```text
PASS: task docs criteria (C1..C6)
PASS: security policy criteria (S1..S5)
PASS: test_toolchain_cli_smoke.py
PASS: test_toolchain_mcp_smoke.py
PASS: ci_suites unit tests (W1..W7)
PASS: ci_service unit tests (X1..X7)
```

## 3. Summary
The Configuration sub-epic (T-00451..T-00460) is verified and closed.
