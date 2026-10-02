# T-02601: Secrets Handling Data Model Research

- **Task**: `T-02601`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Secrets Handling
- **Sub-Epic**: data model (Sub-Epic 1 of 10)
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Context & Objectives
In Phase 2 Security Kernel & Policy Enforcement Point (PEP) Fabric, the Secrets Handling subsystem provides runtime secret management, secure lifecycle control, scoped delegation, and zeroization for agent credentials, API tokens, cryptographic keys, and database connections. This contrasts with earlier static repository scanner tools (`T-00711..T-00760`), establishing runtime kernel-grade secret vaulting and enforcement.

## 2. Research Findings
1. **Separation of Sensitive Material from Metadata**:
   - Secret metadata (ID, name, kind, scope, lifecycle state, version, SHA-256 fingerprint, creation timestamp, expiry) must be freely inspectable and serializable for PEP auditing and access validation.
   - Plaintext secret value (`SecretValue`) must be strictly segregated from general serialization, protected in memory, and zeroized upon `Drop`.
2. **Scoping & Delegation Models**:
   - Hierarchical scopes: `Global`, `Environment(String)`, `Actor(String)`, `Session(String)`.
   - PEP enforcement requires mapping actor context privileges (`PrivilegeLevel`, `PrivilegeContext`) directly to accessible secret scopes.
3. **Lifecycle States**:
   - `Active`: Available for authorized retrieval.
   - `Rotated`: Replaced by a newer version; may permit read-only grace period.
   - `Revoked`: Explicitly invalidated; retrieval attempts are denied.
   - `Expired`: Automatically invalidated when `expires_at <= current_time`.
4. **Security & Zeroization Invariants**:
   - In-memory `SecretValue` must implement manual zeroization on drop (overwriting internal buffers with `0x00`).
   - Constant-time equality checks (`subtle` pattern) to prevent timing side-channel attacks during secret comparison.
   - Masked representation: Expose safe redactions (e.g. first 4 and last 4 characters, or `[REDACTED]`) without leaking underlying entropy.
   - Strict size bounds: Secret names $\le 128$ chars, IDs $\le 64$ chars, payloads $\le 64$ KiB.
