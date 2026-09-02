# T-00410 — Dependency & Toolchain Pinning / recovery & validation: Verification & Evidence

## 1. Verification Overview
This task closes the Recovery & Validation sub-epic (T-00401..T-00410) for Dependency & Toolchain Pinning, completing all tasks through T-00410.

## 2. Test Execution & Evidence

### A. Native Rust Unit Tests (18 Tests in `aiosh-core`)
```text
running 18 tests
test toolchain_config::tests::test_from_path_missing ... ok
test toolchain_config::tests::test_from_path_happy ... ok
test toolchain_config::tests::test_load_toolchain_config_happy_path ... ok
test toolchain_config::tests::test_load_toolchain_config_malformed_json ... ok
test toolchain_config::tests::test_load_toolchain_config_empty_version ... ok
test toolchain_config::tests::test_load_toolchain_config_missing_file ... ok
test toolchain_config::tests::test_to_json_with_sources ... ok
test toolchain_config::tests::test_load_toolchain_config_missing_field ... ok
test toolchain_service::tests::test_check_toolchain_policy_enforcement ... ok
test toolchain_service::tests::test_enforce_toolchain_mismatch_fails ... ok
test toolchain_service::tests::test_collect_toolchain_telemetry_negative_case ... ok
test toolchain_service::tests::test_enforce_toolchain_python_mismatch_fails ... ok
test toolchain_service::tests::test_enforce_toolchain_node_mismatch_fails ... ok
test toolchain_service::tests::test_recover_default_toolchain ... ok
test toolchain_service::tests::test_validate_toolchain_manifest_happy_and_error ... ok
test toolchain_service::tests::test_collect_toolchain_telemetry_captures_details ... ok
test toolchain_service::tests::test_enforce_toolchain_valid ... ok
test toolchain_service::tests::test_reconcile_toolchain_report ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 80 filtered out; finished in 3.16s
```

### B. Documentation Invariants (`tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

### C. Security Policy Invariants (`tools/check_security_policy.py`)
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```

### D. CLI & MCP Smoke Suites
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
The Recovery & Validation sub-epic (T-00401..T-00410) has been fully verified and closed.
