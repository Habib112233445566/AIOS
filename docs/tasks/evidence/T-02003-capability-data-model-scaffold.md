# Task Evidence: T-02003 - Capability Model / data model: Scaffold (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02003`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Scaffold module structure, types, enums, error definitions, and re-exports for the Capability Model data model in `code/aiosh-rust/aiosh-core/src/capability.rs`.

---

## 2. Scaffold Details
- Created `code/aiosh-rust/aiosh-core/src/capability.rs`.
- Defined:
  - `CapabilityRight` (Read, Write, Execute, Delete, Admin, Delegate).
  - `CapabilityScope` (Filesystem, Network, Tool, Process, Ipc, System).
  - `CapabilityConstraints` (not_before, expires_at, max_invocations, quota_bytes).
  - `Capability` core struct with unforgeable SHA-256 ID generation.
  - `CapabilityError` enum and formatted display.
- Registered module in `code/aiosh-rust/aiosh-core/src/lib.rs` and re-exported all core types.
- Verified compilation via `cargo check -p aiosh-core` (Finished dev profile in 28.69s).
