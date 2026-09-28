# T-02413: Sandbox Enforcement Core Service Scaffold

## 1. Objective
Scaffold the core service interfaces and module structure for **Sandbox Enforcement** in `aiosh-core`.

## 2. Changes Made
- Created `code/aiosh-rust/aiosh-core/src/sandbox_service.rs` containing:
  - `HostSandboxCapabilities`: probes native Landlock, Seccomp-BPF, no_new_privs, and platform identity.
  - `SandboxConfig`: configuration for default profiles, capture byte caps, PEP enforcement, and audit settings.
  - `SandboxService`: lifecycle management for profiles (`register_profile`, `get_profile`, `list_profiles`, `remove_profile`), host capability probing, command execution, and audit emission.
  - Error constants: `ERR_SANDBOX_PROFILE_NOT_FOUND`, `ERR_SANDBOX_PROFILE_EXISTS`, `ERR_SANDBOX_CANNOT_DELETE_DEFAULT`, `ERR_SANDBOX_PEP_UNAUTHORIZED`, `ERR_SANDBOX_EXEC_FAILED`.
- Updated `code/aiosh-rust/aiosh-core/src/lib.rs`:
  - Added `pub mod sandbox_service;`.
  - Re-exported `HostSandboxCapabilities`, `SandboxConfig`, `SandboxService`, and related constants.

## 3. Verification
Confirmed workspace builds cleanly across all crates without compiler errors or warnings.
