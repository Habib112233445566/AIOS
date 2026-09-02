# T-00266 — Automated Tests: Integration

## Integration Scope
Integrate the automated tests for Release Packaging & Backup into the standard CI and validation pathways.

## Implementation
- **CI Test Suite Registration**: The tests written in `T-00264` reside in `aiosh-core/src/release_config.rs`. Because they are standard Rust `#[test]` modules, they are automatically discovered and executed by `cargo test --workspace`.
- **Smoke Suite Invocation**: The `rust_smoke` step in `tools/ci_suites.py` invokes `cargo test`. Thus, the configuration loader hardening tests are officially wired into the `ci/run_all_smokes.sh` continuous integration pathway.
- **Cross-Substrate Parity**: The configuration JSON format (`release.json`) being parsed in Rust is conceptually decoupled from Python, but tests ensure it rejects malicious cross-substrate inputs.

## Validation
- The tests ran and passed natively in the previous tasks.
- No new CLI or MCP surfaces are required for tests. They are completely internal and trigger automatically during CI validations.

Task is complete.
