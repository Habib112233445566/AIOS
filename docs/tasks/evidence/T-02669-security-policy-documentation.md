# T-02669: Secrets Handling Security Policy Documentation

- **Task**: `T-02669`
- **Sub-Epic**: Phase 2 — Security Kernel & PEP Fabric / Secrets Handling / security policy
- **Date**: 2026-10-03
- **Status**: PASSED

## 1. Documentation Overview
Authoritative documentation for the Secrets Handling security policy subsystem has been updated and synchronized in `docs/SPEC-SECRETS-POLICY.md`:
1. **Specification & Schema (`docs/SPEC-SECRETS-POLICY.md`)**:
   - Documented `SecretSecurityPolicy` schema, modes (`enforcing`, `permissive`, `disabled`), and verdicts.
   - Listed all error constants (`SECPOL_ERR_GLOBAL_DISALLOWED`, `SECPOL_ERR_KIND_PROHIBITED`, `SECPOL_ERR_PAYLOAD_TOO_LARGE`, `SECPOL_ERR_EXPOSE_REQUIRED`, `SECPOL_ERR_DENIED`, `SECPOL_ERR_VALIDATION`).
   - Detailed complete environment variable overrides (`AIOS_SECRETS_POLICY_PATH`, `AIOS_SECRETS_POLICY_MODE`, `AIOS_SECRETS_POLICY_DISALLOW_GLOBAL`, `AIOS_SECRETS_POLICY_MAX_PAYLOAD`, `AIOS_SECRETS_POLICY_REQUIRE_EXPOSE`).
2. **Operational Invocation Examples**:
   - Copy-pasteable CLI commands for `aiosh secret policy show`, `check`, and `set-mode`.
   - Copy-pasteable MCP tool request schemas for `aios.secret.policy` (`show`, `check`, `set-mode`).
3. **Honest Constraints & Limitations**:
   - 64 KiB file size ceiling (`MAX_SECRET_SECURITY_POLICY_BYTES`).
   - Directory traversal (`..`) and control character rejection.
   - Cap of 64 prohibited kinds.
   - Atomic temporary-file rename persistence.
4. **Codebase Rustdoc**:
   - Maintained complete rustdoc comments across all types and methods in `code/aiosh-rust/aiosh-core/src/secret_policy.rs`.
