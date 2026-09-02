# T-00405 — Dependency & Toolchain Pinning / recovery & validation: Unit Test

## 1. Unit Test Scope
This task tests the Recovery & Validation functionality for Dependency & Toolchain Pinning in `aiosh-core`.

## 2. Test Coverage & Execution
- **Module**: `code/aiosh-rust/aiosh-core/src/toolchain_service.rs`
- **Tests**:
  1. `test_validate_toolchain_manifest_happy_and_error`:
     - Positive test: Valid configuration file parses cleanly and populates `ToolchainManifest`.
     - Negative test: Non-existent path returns explicit error.
     - Negative test: Malformed JSON payload returns parsing error.
  2. `test_recover_default_toolchain`:
     - Asserts that default fallback recovery yields the canonical compile-time pinned versions (`rust=1.99.0`, `python=3.14`, `node=v24.18`).
  3. `test_reconcile_toolchain_report`:
     - Asserts that conforming host toolchains produce `is_conforming: true` with `"conforming"` status across components.
     - Asserts that drifted host toolchains produce `is_conforming: false`, `"drifted"` status, and actionable remediation steps.

## 3. Test Output
```text
running 10 tests
test toolchain_service::tests::test_check_toolchain_policy_enforcement ... ok
test toolchain_service::tests::test_enforce_toolchain_mismatch_fails ... ok
test toolchain_service::tests::test_collect_toolchain_telemetry_negative_case ... ok
test toolchain_service::tests::test_enforce_toolchain_node_mismatch_fails ... ok
test toolchain_service::tests::test_enforce_toolchain_python_mismatch_fails ... ok
test toolchain_service::tests::test_recover_default_toolchain ... ok
test toolchain_service::tests::test_validate_toolchain_manifest_happy_and_error ... ok
test toolchain_service::tests::test_collect_toolchain_telemetry_captures_details ... ok
test toolchain_service::tests::test_enforce_toolchain_valid ... ok
test toolchain_service::tests::test_reconcile_toolchain_report ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 3.27s
```
