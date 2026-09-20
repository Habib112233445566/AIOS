# Task Evidence: T-01893 - Network Bootstrap / recovery & validation: Scaffold

## 1. Overview
- **Task ID**: `T-01893`
- **Sub-Epic**: 10 (Network Bootstrap Recovery & Validation)
- **Goal**: Scaffold data structures, validation logic, and error handlers for Network Recovery & Validation in `network_recovery.rs`.

---

## 2. Scaffolding Scope
1. Scaffolded `code/aiosh-rust/aiosh-core/src/network_recovery.rs`:
   - Defined constants: `MAX_NETWORK_STORE_SIZE` (1 MB), `MAX_RECOVERY_ISSUES` (100).
   - Defined error codes: `NVAL_PATH_ERROR`, `NVAL_IO_ERROR`, `NVAL_VALIDATION_ERROR`, `NVAL_PARSE_ERROR`.
   - Implemented `validate_network_store_path`: enforces non-empty, $\le 1024$ chars, valid UTF-8, no control chars, no `..`, and `.json` extension.
   - Defined `NetworkValidationReport`, implementing `validate_invariants()` checking `NVAL1..NVAL4`.
   - Defined `NetworkRecoveryAction` and `NetworkRecoveryReport`.
   - Implemented `validate_network_state()` evaluating interface validity, loopback presence, default route existence, dangling route references, and DNS configuration.
   - Implemented `check_network_file()` reading, size checking, JSON parsing, and validating disk files.
2. Re-exported module and public types in `code/aiosh-rust/aiosh-core/src/lib.rs`.
3. Verified clean compilation via `cargo check` (0 errors, 0 warnings).

Status: Scaffolding completed. Ready for full implementation in `T-01894`.
