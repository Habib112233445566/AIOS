# T-02661: Secrets Handling Security Policy Research

- **Task**: `T-02661`
- **Sub-Epic**: Phase 2 — Security Kernel & PEP Fabric / Secrets Handling / security policy
- **Date**: 2026-10-03
- **Status**: PASSED

## 1. Research Objectives
Establish architectural facts, policy vectors, governance invariants, and prior art for declarative security policy enforcement in the Secrets Handling subsystem (`code/aiosh-rust/aiosh-core/src/secret_policy.rs`).

## 2. Prior Art & Subsystem Analysis
Across the AIOS kernel and security architecture (`privilege_policy.rs`, `pep_grant_security_policy.rs`, `sandbox_policy.rs`):
1. **Tri-State Policy Modes**:
   - `Enforcing`: Fail-closed evaluation. Disallowed operations are rejected immediately with specific error codes.
   - `Permissive`: Evaluates policy and permits requests but emits warning records to the audit ring.
   - `Disabled`: Diagnostic/maintenance bypass.
2. **Explicit Verdict Model**:
   - `Permit`: Access allowed.
   - `PermitWithWarning { warning }`: Access allowed with telemetry annotation.
   - `Deny { reason, code }`: Access blocked fail-closed.
3. **Declarative Rule Vectors for Secrets Handling**:
   - `disallow_global_secrets`: Prevents storing secrets in global scope to eliminate broadcast leakage risk.
   - `max_payload_bytes`: Caps individual secret payload sizes.
   - `prohibited_kinds`: Prohibits registration of specific high-risk secret kinds (e.g. unencrypted PrivateKey) in non-isolated scopes.
   - `require_expose_flag`: Forbids programmatic raw secret retrieval unless explicitly requested.
   - `max_lifetime_seconds`: Enforces maximum TTL on vaulted secrets.
4. **Integration Surface**:
   - CLI: `aiosh secret policy <show|check|validate>`
   - MCP: `aios.secret.policy` tool
   - Core: Evaluated directly during `store_secret`, `get_secret`, and `rotate_secret` in `SecretService`.

## 3. Key Decisions for Specification (T-02662)
1. Author `docs/SPEC-SECRETS-POLICY.md` specifying data model, defaults, and evaluation logic.
2. Scaffold `code/aiosh-rust/aiosh-core/src/secret_policy.rs` with `SecretSecurityPolicy`, `SecretPolicyMode`, and `SecretPolicyVerdict`.
3. Wire policy evaluation into `SecretService` and surface via CLI and MCP.
