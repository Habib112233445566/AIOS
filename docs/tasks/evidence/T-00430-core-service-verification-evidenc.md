# T-00430 — Documentation Index Control / core service: Verification & Evidence

## 1. Verification Overview
This task concludes the Core Service sub-epic (T-00421..T-00430) for Documentation Index Control in `aiosh-core`.

## 2. Test Execution & Evidence

### A. Native Rust Unit Tests (`cargo test -p aiosh-core doc_index`)
```text
running 16 tests
test doc_index::tests::test_doc_index_manifest_default_is_valid ... ok
test doc_index::tests::test_doc_index_manifest_duplicate_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_path_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_section_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_title_fails ... ok
test doc_index::tests::test_doc_index_manifest_empty_version_fails ... ok
test doc_index::tests::test_doc_index_manifest_links_limit_fails ... ok
test doc_index::tests::test_doc_index_manifest_malformed_json_fails ... ok
test doc_index::tests::test_doc_index_manifest_query_helpers ... ok
test doc_index::tests::test_doc_index_manifest_roundtrip_happy ... ok
test doc_index_service::tests::test_parse_markdown_links_happy ... ok
test doc_index_service::tests::test_build_doc_index_missing_file_error ... ok
test doc_index_service::tests::test_parse_markdown_title_happy ... ok
test doc_index_service::tests::test_real_repo_docs_index_and_validation ... ok
test doc_index_service::tests::test_validate_doc_links_escape_detected ... ok
test doc_index_service::tests::test_validate_doc_links_and_build_index ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.11s
```

### B. Documentation Invariants (`python tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

### C. Security Policy Invariants (`python tools/check_security_policy.py`)
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```

### D. System Smokes and CI Suites
```text
PASS: aiosh toolchain show
PASS: aiosh toolchain check
PASS: aiosh toolchain custom config valid
PASS: aiosh toolchain invalid subcommand
PASS: aiosh toolchain missing config negative test
PASS: aiosh toolchain corrupted config negative test
PASS: aiosh toolchain version mismatch negative test
PASS: test_toolchain_cli_smoke.py

PASS: aios.toolchain.config.get
PASS: aios.toolchain.check
PASS: aios.toolchain unknown tool negative test
PASS: test_toolchain_mcp_smoke.py

PASS: ci_suites unit tests (W1..W7)
PASS: ci_service unit tests (X1..X7)
```

## 3. Summary
The Core Service sub-epic (T-00421..T-00430) is verified and closed.
