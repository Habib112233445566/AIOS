# T-02403: Sandbox Enforcement Data Model Scaffold

## 1. Objective
Scaffold the core data model structures and interfaces for **Sandbox Enforcement** within `aiosh-core` without runtime errors, ensuring clean type-level contracts and export linkage across crates.

## 2. Changes Made
- Created `code/aiosh-rust/aiosh-core/src/sandbox_data_model.rs` defining:
  - Enums: `SandboxProfileType`, `IsolationLevel`, `NetworkIsolationMode`, `SyscallAction`, `SandboxExecutionStatus`.
  - Structs: `ResourceLimits`, `FilesystemPolicy`, `SyscallPolicy`, `EnvironmentPolicy`, `SandboxProfile`, `SandboxExecutionRequest`, `SandboxComponentStatus`, `SandboxExecutionResult`.
  - Boundary constants: `MIN_MEMORY_BYTES`, `MAX_MEMORY_BYTES`, `MIN_WALL_TIME_MS`, `MAX_WALL_TIME_MS`, `MAX_PROCESSES_LIMIT`, `MAX_OPEN_FILES_LIMIT`.
  - Error constants: `ERR_SANDBOX_INVALID_PATH`, `ERR_SANDBOX_INVALID_LIMIT`, `ERR_SANDBOX_POLICY_CONFLICT`, `ERR_SANDBOX_EMPTY_COMMAND`.
  - Standard, Strict, and Permissive profile factories (`standard()`, `strict()`, `permissive()`).
  - Validation routines and legacy bridge methods (`to_legacy_policy()`, `from_legacy_policy()`).
- Updated `code/aiosh-rust/aiosh-core/src/lib.rs`:
  - Declared `pub mod sandbox_data_model;`.
  - Re-exported data model types in the public crate root.

## 3. Verification
Verified compilation via `cargo check --workspace` producing 0 errors and 0 warnings.
