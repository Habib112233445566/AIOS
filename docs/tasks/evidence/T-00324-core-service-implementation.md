# T-00324 — Dependency & Toolchain Pinning: core service Implementation

## 1. Overview
This task implemented the minimal working behavior for the `toolchain_service` core module, responsible for enforcing the constraints defined in the `ToolchainManifest`.

## 2. Implementation Details
- **Module**: `aiosh-core/src/toolchain_service.rs`
- **Functionality**:
  - `enforce_toolchain(&ToolchainManifest) -> Result<(), String>` executes `rustc -V`, `python3 -V` (fallback to `python -V`), and `node -v` (if required).
  - Validates that the active environment version string contains the pinned version.
  - Halts with descriptive string errors if binaries are missing or versions mismatch.
- **Unit Testing**:
  - Added a basic failing unit test `test_enforce_toolchain_mismatch_fails` inline with the implementation, which guarantees the service returns an error when given impossible version requirements (e.g. `999.99.99`).

## 3. Adherence to Strictures
- **Reuse**: The implementation reuses the `ToolchainManifest` data model and uses the standard library `std::process::Command` to invoke system processes.
- **No New Dependencies**: The implementation does not add any third-party crates; it relies entirely on the Rust standard library.
- **Audit Logging**: As a core service module, it simply returns a `Result`. Downstream tasks that use this (like task execution or building releases) will handle mapping these errors to proper `task.ledger` or `audit.log` entries via the existing dispatcher.

## 4. Verification
- `cargo test` confirms the new unit test passes.
- No existing smoke suites or unit tests failed (87 tests passed).
