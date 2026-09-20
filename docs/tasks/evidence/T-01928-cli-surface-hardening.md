# Task Evidence: T-01928 - System Update / CLI surface: Hardening

- **Task**: `T-01928`
- **Sub-Epic**: `Sub-Epic 3: Operator CLI & Control Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Hardened the System Update operator CLI control surface in `code/aiosh-rust/aiosh-cli/src/main.rs`:
- **Manifest Path Bounds**: Added strict length limit (`len <= 1024`) and control character rejection (`PATH_CONTAINS_CONTROL_CHAR`) on the manifest file path argument in `check`.
- **Manifest File Quota**: Added file metadata size check before reading into memory, capping manifest file sizes at 1MB (`1_048_576` bytes) to prevent memory allocation denial-of-service (`MANIFEST_TOO_LARGE`).
- **Version String Sanitization**: Added length boundary (`len <= 64`) and control character rejection (`INVALID_VERSION_STRING`) on the confirmed version parameter in `confirm`.
- **Audit Emissions**: All rejected inputs trigger security classification audit events before returning exit code 2.
- **Unit Testing**: Added dedicated test cases in `update_cli_tests` exercising manifest path length and control characters, and version string bounds.

## Verification
- Executed `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli update_cli_tests`.
- 5/5 unit tests passed cleanly in 0.62s.
