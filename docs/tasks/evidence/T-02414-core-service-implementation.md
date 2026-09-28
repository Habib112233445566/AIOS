# T-02414: Sandbox Enforcement Core Service Implementation

## 1. Implementation Overview
Implemented the **Sandbox Enforcement Core Service** in `code/aiosh-rust/aiosh-core/src/sandbox_service.rs` as the central execution and lifecycle coordinator for process isolation, policy enforcement, and audit integration.

## 2. Core Service Capabilities
1. **Host Capabilities Discovery**:
   - `HostSandboxCapabilities::probe()` dynamically inspects the OS kernel for Landlock LSM ABI version, Seccomp-BPF filters, and `no_new_privs` flags.
2. **Profile Catalog Management**:
   - Maintains thread-safe in-memory profile registry.
   - Pre-populates default `standard`, `strict`, and `permissive` profiles.
   - Protects core profiles against accidental removal while allowing registration, lookup, and deletion of custom user/agent profiles.
3. **Supervised Process Execution**:
   - Executes commands with working directory resolution and environment sanitization based on profile rules (`clean_env`, variable allowlist, injected vars).
   - Captures output with deterministic truncation limits (`max_output_capture_bytes`).
4. **Security Gating & Auditing**:
   - Implements PEP authorization gating (`enforce_pep_grants` requiring active `pep_grant_id`).
   - Automatically writes an extended cryptographic audit record to `AuditRing` upon execution completion, including command, profile, exit status, duration, and telemetry metadata.
   - Embeds the cryptographic audit row hash into `SandboxExecutionResult.audit_hash`.

## 3. Verification
Implemented targeted unit tests verifying profile catalog management, capability probing, un-audited execution, audited execution, and PEP authorization gating.
