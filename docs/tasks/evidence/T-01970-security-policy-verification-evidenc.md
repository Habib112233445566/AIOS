# Task Evidence: T-01970 (System Update / security policy: Verification & Evidence)

## Summary
Formally verified and closed Sub-Epic 7 ("System Update Security Policy", tasks T-01961 through T-01970).
All policy invariants (UPOL1 - UPOL6), execution modes, boundary conditions, and cross-substrate parity checks have been validated.

## Verification Matrix & Results

| Component | Target / Test Suite | Executed Command | Result |
|---|---|---|---|
| Rust Policy Unit Tests | `aiosh-core::test_system_update_policy` | `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_policy` | **PASS** (9/9 tests in 0.01s) |
| Python / MCP Integration Smoke | `test_system_update_policy_smoke.py` | `python code/aiosh-mcp/tests/test_system_update_policy_smoke.py` | **PASS** (7/7 check suites pass) |
| Invariant Verification | Invariants UPOL1 - UPOL6 | Verified via test suite (`test_upol1` through `test_upol6`) | **PASS** |
| Hardened Path & Symlink Defense | `from_file` / `save_to_file` symlink rejection | Verified via unit tests | **PASS** |
| Documentation Completeness | Section 10 of `docs/system_update.md` | Verified present and exhaustive | **PASS** |

## Sub-Epic 7 Lifecycle Sign-off
- **T-01961**: Research completed
- **T-01962**: Specification completed
- **T-01963**: Scaffold completed
- **T-01964**: Implementation completed
- **T-01965**: Unit Test completed
- **T-01966**: Integration completed
- **T-01967**: Security Review completed
- **T-01968**: Hardening completed
- **T-01969**: Documentation completed
- **T-01970**: Verification & Evidence completed (Sub-Epic 7 Closed)
