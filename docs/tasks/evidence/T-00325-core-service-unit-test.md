# T-00325 — Dependency & Toolchain Pinning: core service Unit Test

## 1. Overview
This task adds focused automated tests for the `toolchain_service` core module to verify enforcement of toolchain manifests against the host environment.

## 2. Implementation
- **Module**: `code/aiosh-rust/aiosh-core/src/toolchain_service.rs`
- **Tests Added**:
  - `test_enforce_toolchain_valid`: Tests valid inputs by running the real `rustc` and `python` binaries to fetch the current version and asserts that `enforce_toolchain` passes.
  - `test_enforce_toolchain_mismatch_fails`: Tests invalid Rust input (`999.99.99`).
  - `test_enforce_toolchain_python_mismatch_fails`: Tests invalid Python input (`Python 999.99.99`).
  - `test_enforce_toolchain_node_mismatch_fails`: Tests optional Node invalid input/boundary (`v999.99.99`).

## 3. Verification
The unit tests cover valid input, invalid input, boundary values, and primary failure modes.

```
running 4 tests
test toolchain_service::tests::test_enforce_toolchain_python_mismatch_fails ... ok
test toolchain_service::tests::test_enforce_toolchain_mismatch_fails ... ok
test toolchain_service::tests::test_enforce_toolchain_valid ... ok
test toolchain_service::tests::test_enforce_toolchain_node_mismatch_fails ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 86 filtered out; finished in 1.62s
```
