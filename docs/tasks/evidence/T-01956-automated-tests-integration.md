# Task Evidence: T-01956 (System Update / automated tests: Integration)

## Summary
Integrated the automated test suite for the System Update Mechanism with the surrounding system:
1. `code/aiosh-rust/aiosh-core/tests/test_system_update_e2e.rs`: Integrated into Cargo test runner (`cargo test -p aiosh-core --test test_system_update_e2e`), passing 9/9 tests in 0.02s.
2. `code/aiosh-mcp/tests/test_system_update_e2e_smoke.py`: End-to-end Python integration smoke suite validating real lifecycle staging, digest calculation, fault injection, and slot rollback.
3. Cross-substrate parity: Verified that JSON representations of slot states, update states, and manifests match across Rust core and Python/MCP environments.

## Integration Test Results
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_e2e`: **PASS** (9/9 passed in 0.02s)
- `python code/aiosh-mcp/tests/test_system_update_e2e_smoke.py`: **PASS** (3/3 check suites pass)
- `python code/aiosh-cli/tests/test_system_update_smoke.py`: **PASS** (5/5 tests pass)
