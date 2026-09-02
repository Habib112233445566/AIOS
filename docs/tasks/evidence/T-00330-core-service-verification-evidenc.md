# T-00330 — Dependency & Toolchain Pinning: core service Verification & Evidence

## 1. Overview
This task concludes the Dependency & Toolchain Pinning epic by verifying all new components across the repository and capturing successful test results.

## 2. Test Execution
All test suites across the Rust shipping path (`aiosh-core`, `aiosh-cli`, `aiosh-sandbox`, `aiosh-mcp`) were executed.

**Command:**
```bash
cargo test --all
```

**Results Summary:**
- `aiosh-cli`: 13 tests passed.
- `aiosh-core`: 90 tests passed.
- `aiosh-mcp`: 0 tests (placeholder).
- `aiosh-sandbox`: 0 tests (placeholder).

**Test Output:**
```
     Running unittests src\main.rs (target\debug\deps\aiosh-8814d271a6af31ad.exe)

running 13 tests
...
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\aiosh_core-04423105fe4b6d57.exe)

running 90 tests
...
test toolchain_config::tests::test_load_toolchain_config_happy_path ... ok
test toolchain_config::tests::test_load_toolchain_config_empty_version ... ok
test toolchain_service::tests::test_enforce_toolchain_mismatch_fails ... ok
test toolchain_service::tests::test_enforce_toolchain_valid ... ok
...
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.71s
```

## 3. Milestone Completion
The Dependency & Toolchain Pinning epic (T-00322 through T-00330) is complete. The system now guarantees that agent actions relying on ecosystem binaries (`rustc`, `python3`, `node`) will fail-fast with a loud, audited refusal if the host OS environments drift from the explicitly pinned manifest.
