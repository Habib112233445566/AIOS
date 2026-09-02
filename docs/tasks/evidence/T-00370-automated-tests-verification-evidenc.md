# T-00370 — Dependency & Toolchain Pinning / automated tests: Verification & Evidence

## 1. Verification Scope
This task closes the Dependency & Toolchain Pinning epic's automated test sequence (T-00361 .. T-00370) by executing the complete test matrix across native Rust crates and cross-substrate Python CI smoke suites.

## 2. Test Execution & Evidence

### A. Native Rust Unit Tests (`aiosh-core` toolchain tests)
```text
running 12 tests
test toolchain_config::tests::test_from_path_missing ... ok
test toolchain_config::tests::test_from_path_happy ... ok
test toolchain_config::tests::test_load_toolchain_config_happy_path ... ok
test toolchain_config::tests::test_load_toolchain_config_empty_version ... ok
test toolchain_config::tests::test_load_toolchain_config_missing_file ... ok
test toolchain_config::tests::test_load_toolchain_config_missing_field ... ok
test toolchain_config::tests::test_load_toolchain_config_malformed_json ... ok
test toolchain_config::tests::test_to_json_with_sources ... ok
test toolchain_service::tests::test_enforce_toolchain_mismatch_fails ... ok
test toolchain_service::tests::test_enforce_toolchain_python_mismatch_fails ... ok
test toolchain_service::tests::test_enforce_toolchain_valid ... ok
test toolchain_service::tests::test_enforce_toolchain_node_mismatch_fails ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 80 filtered out; finished in 5.88s
```

### B. CLI and MCP Smoke Test Suites
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
```

### C. CI Orchestrator & Documentation Invariants
```text
PASS: ci_suites unit tests (W1..W7)
PASS: ci_service unit tests (X1..X7)
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts
PASS: task docs criteria (C1..C6)
```

## 3. Verdict
All relevant unit, smoke, integration, and documentation tests pass cleanly. The Dependency & Toolchain Pinning automated test sequence is complete.
