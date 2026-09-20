# Task Evidence: T-02008 - Capability Model / data model: Hardening (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02008`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Harden the Capability Model data model against injection attacks, path traversal, timestamp spoofing, and unconstrained delegation.

---

## 2. Hardened Surface & Defense Implementations

1. **Identifier Validation (`validate_identifier`)**:
   - Enforces length bounds ($1..=128$).
   - Rejects control characters, NUL bytes, whitespace, and any characters outside `[a-zA-Z0-9_\-:\.]`.
   - Protects both `issuer` and `subject` in `new()` and `attenuate()`.

2. **Resource Scope & Path Hygiene (`validate_scope`)**:
   - Filesystem: validates length $\le 1024$, ensures paths are absolute, rejects control characters, and blocks `..` parent directory traversal components.
   - Network: bounds host length $\le 255$ and protocol $\le 32$.
   - Tool: bounds tool name $\le 128$ and caps allowed actions $\le 64$.
   - Process / IPC / System: bounds executable/channel/subsystem identifiers.

3. **Temporal Bounds & Quota Validation (`validate_constraints`)**:
   - Validates RFC3339 timestamp compliance for `not_before` and `expires_at` at creation/attenuation time.
   - Enforces that `not_before <= expires_at`.

4. **Attenuation Invariant Preservation (`CAP3`)**:
   - Enforces that derived child capabilities pass all input validation, scope validation, constraint monotonicity, and right subset checks.

---

## 3. Verification
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_capability_data_model`: 6/6 tests passed in 0.00s.
