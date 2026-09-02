# T-00390 — Dependency & Toolchain Pinning / observability: Verification & Evidence

## 1. Verification Scope
This task closes the Dependency & Toolchain Pinning observability sequence (T-00381 .. T-00390) with verified test evidence across all native Rust crates, Python CI harnesses, and documentation suites.

## 2. Test Execution Evidence

### A. Core Rust Toolchain Unit Tests (15 Tests)
```text
running 15 tests
test toolchain_config::tests::test_from_path_missing ... ok
test toolchain_config::tests::test_load_toolchain_config_empty_version ... ok
test toolchain_config::tests::test_load_toolchain_config_happy_path ... ok
test toolchain_config::tests::test_load_toolchain_config_malformed_json ... ok
test toolchain_config::tests::test_from_path_happy ... ok
test toolchain_config::tests::test_load_toolchain_config_missing_file ... ok
test toolchain_config::tests::test_to_json_with_sources ... ok
test toolchain_service::tests::test_check_toolchain_policy_enforcement ... ok
test toolchain_config::tests::test_load_toolchain_config_missing_field ... ok
test toolchain_service::tests::test_enforce_toolchain_mismatch_fails ... ok
test toolchain_service::tests::test_collect_toolchain_telemetry_negative_case ... ok
test toolchain_service::tests::test_enforce_toolchain_python_mismatch_fails ... ok
test toolchain_service::tests::test_enforce_toolchain_node_mismatch_fails ... ok
test toolchain_service::tests::test_collect_toolchain_telemetry_captures_details ... ok
test toolchain_service::tests::test_enforce_toolchain_valid ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 80 filtered out; finished in 2.31s
```

### B. Documentation Invariants & Security Policy
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts
PASS: task docs criteria (C1..C6)

[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist
PASS: security policy criteria (S1..S5)
```

### C. CLI & MCP Smoke Suites & CI Invariants
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

## 3. Verdict
All unit, integration, observability, and regression tests pass cleanly. The Dependency & Toolchain Pinning observability sequence is verified and complete.
