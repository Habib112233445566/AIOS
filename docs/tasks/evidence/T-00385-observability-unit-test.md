# T-00385 — Dependency & Toolchain Pinning / observability: Unit Test

## 1. Unit Test Objectives
Add focused automated tests for the observability and runtime telemetry collection of Dependency & Toolchain Pinning in `aiosh-core`.

## 2. Test Execution & Coverage
- **Module Under Test**: `code/aiosh-rust/aiosh-core/src/toolchain_service.rs`
- **Test Cases**:
  1. `test_collect_toolchain_telemetry_captures_details`: Validates happy-path telemetry collection, ensuring host `rustc` and `python` version outputs are non-empty and `check_passed` is `true`.
  2. `test_collect_toolchain_telemetry_negative_case`: Validates failure-mode behavior when supplied with impossible/mismatched version constraints (`999.99.99`), verifying that binary stdout is captured while `check_passed` evaluates to `false`.

## 3. Test Output
```text
running 2 tests
test toolchain_service::tests::test_collect_toolchain_telemetry_negative_case ... ok
test toolchain_service::tests::test_collect_toolchain_telemetry_captures_details ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 93 filtered out; finished in 6.25s
```
