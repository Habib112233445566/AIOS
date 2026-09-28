# T-02404: Sandbox Enforcement Data Model Implementation

## 1. Implementation Scope & Summary
Fully implemented the **Sandbox Enforcement Data Model** in `code/aiosh-rust/aiosh-core/src/sandbox_data_model.rs`, providing robust type-safe profiles, resource bounds, filesystem isolation rules, syscall policies, environment filters, execution requests, and result envelopes.

## 2. Key Components Added
1. **Enums & Structs**:
   - `SandboxProfileType` (`Strict`, `Standard`, `Permissive`, `IsolatedDev`, `Custom`).
   - `IsolationLevel` (`ProcessOnly`, `RestrictedNamespaces`, `FullLandlockSeccomp`).
   - `ResourceLimits`: Enforces bounded memory (1MB..64GB), wall-clock execution limits (10ms..1h), process counts (1..4096), and open file descriptors (16..65536).
   - `FilesystemPolicy`: Manages `paths_ro`, `paths_rw`, `paths_execute`, `paths_denied`, with path traversal (`..`) detection and mutual exclusion validation.
   - `NetworkIsolationMode`: `Disabled`, `LoopbackOnly`, `FilteredEgress`, `Unrestricted`.
   - `SyscallPolicy` & `SyscallAction`: Standardized 17-syscall default denylist, allowlist extensions, and configurable trigger action (`KillProcess`, `ReturnErrno`, `Log`, `Allow`).
   - `EnvironmentPolicy`: Host environment stripping (`clean_env`), variable pass-through allowlist, and sanitized variable injection.
   - `SandboxProfile`: Unified profile with `standard()`, `strict()`, `permissive()` factory constructors and `canonical_hash()` computation.
   - `SandboxProfileBuilder`: Fluent, safe builder pattern for constructing valid custom profiles.
   - `SandboxExecutionRequest` & `SandboxExecutionResult`: Strongly-typed command dispatch, PEP grant integration (`pep_grant_id`, `session_id`), and execution result telemetry.
2. **Legacy Bridge**:
   - `to_legacy_policy()` and `from_legacy_policy()` providing full backwards compatibility with Sprint 2 `SandboxPolicy`.

## 3. Invariant Verification
- Verified path validation rejects traversal `..` and conflicting permissions.
- Verified deterministic canonical JSON hashing.
