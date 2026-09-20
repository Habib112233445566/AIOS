# Task Evidence: T-02004 - Capability Model / data model: Implementation (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02004`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Implement the core data model methods, monotonic attenuation, quota consumption, scope matching, and revocation for `Capability` in `code/aiosh-rust/aiosh-core/src/capability.rs`.

---

## 2. Implemented Capabilities & Invariants

1. **Unforgeable ID & Lifecycle**:
   - `Capability::new()` generates unforgeable identifiers utilizing SHA-256 over issuer, subject, and timestamp.
2. **Rights Verification (`check_right`, `has_right`)**:
   - Explicit check against granted rights enum (`Read`, `Write`, `Execute`, `Delete`, `Admin`, `Delegate`).
3. **Temporal Validity & Quotas (`check_validity_at`, `consume_invocation`, `consume_bytes`)**:
   - Validates ISO-8601 RFC3339 `not_before` and `expires_at` boundaries.
   - Saturated tracking and quota enforcement on invocations and byte consumption.
4. **Scope Matching (`matches_scope`)**:
   - Filesystem: recursive directory hierarchy matching and exact path equivalence.
   - Network: wildcard host, port, and protocol matching.
   - Tool: tool identifier and allowed actions allowlist matching.
   - Process: executable name matching and memory ceiling enforcement.
5. **Monotonic Attenuation (`attenuate`)**:
   - Mandates `CapabilityRight::Delegate` on the parent.
   - Validates that all child rights exist within parent rights (no privilege escalation).
   - Enforces scope confinement (child scope cannot exceed parent scope).
   - Enforces constraint monotonicity (child expiration and quotas cannot exceed parent remaining budget).
   - Tracks lineage via `parent_id`.
6. **Revocation (`revoke`)**:
   - Immediately sets `revoked = true`, invalidating future invocations.

---

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core`: Passed in 7.52s.
