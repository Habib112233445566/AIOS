# T-02611: Secrets Handling Core Service Research

- **Task**: `T-02611`
- **Sub-Epic**: Secrets Handling / core service (Sub-Epic 2 of 10)
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Objectives & Context
The Core Service for Secrets Handling (`SecretService`) acts as the runtime vault and authorization gate in Phase 2 Security Kernel & PEP Fabric. It manages the registration, storage, scoped access, rotation, revocation, and atomic disk persistence of `SecretEntry` instances while ensuring fail-closed security invariants.

## 2. Research Findings & Architectural Decisions
1. **Scoped Isolation & Policy Gates**:
   - Access to a secret must verify `caller_scope.allows(&secret.scope)`. For example, secrets scoped to an actor `Actor("agent_a")` or environment `Environment("prod")` must deny callers lacking matching scope or global authority.
2. **State & Expiration Handling**:
   - Secrets marked `Revoked` or past their `expires_at` timestamp are inaccessible for runtime payload disclosure (`SECSVC_ERR_INACCESSIBLE`), returning fail-closed errors.
3. **Storage & Atomic Persistence**:
   - Vault files stored on disk default to `~/.aiosh/secrets_vault.json`.
   - All disk mutations must utilize atomic temp-file write (`.tmp.<pid>`) and rename semantics (`std::fs::rename`) to guard against corruption during sudden power loss or process kill.
   - Store size bounded at 1 MiB (`MAX_SECRETS_STORE_SIZE`) and capacity bounded at 1,024 secrets (`MAX_SECRETS_VAULT_CAPACITY`) to prevent resource exhaustion attacks.
4. **Separation of Metadata and Payload**:
   - Listing operations (`list_secrets`) return only sanitized `SecretMetadata`, ensuring secret material is never exposed during audit or enumeration tasks.
