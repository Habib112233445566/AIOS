# Task T-02493 Evidence: Sandbox Recovery & Validation Scaffold

## Goal
Scaffold the types, error constants, interfaces, and module exports for Sandbox Enforcement Recovery & Validation.

## Delivered Artifacts
1. **Module Implementation (`code/aiosh-rust/aiosh-core/src/sandbox_recovery.rs`)**:
   - Defined `SandboxValidationSeverity` (`Error`, `Warning`).
   - Defined `SandboxValidationIssue` and `SandboxValidationReport`.
   - Defined `SandboxRecoveryStrategy` (`DryRun`, `RestoreFactoryDefaults`, `QuarantineAndReset`).
   - Defined `SandboxRecoveryResult`.
   - Scaffolded `SandboxRecoveryManager` with `validate()`, `validate_profile_file()`, and `recover()`.
   - Defined error codes: `SANDBOXRECV_ERR_IO`, `SANDBOXRECV_ERR_TRAVERSAL`, `SANDBOXRECV_ERR_CORRUPT`, `SANDBOXRECV_ERR_MISSING_FACTORY`, `SANDBOXRECV_ERR_LIMIT_BOUNDS`, and `MAX_PROFILE_FILE_BYTES`.
2. **Library Exports (`code/aiosh-rust/aiosh-core/src/lib.rs`)**:
   - Added `pub mod sandbox_recovery;`.
   - Re-exported core symbols and error codes.
3. **Compilation**:
   - `cargo check --workspace` clean across all crates with 0 warnings, 0 errors.
