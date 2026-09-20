# Task Evidence: T-02013 - Capability Model / core service: Scaffold (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02013`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Create module skeleton, types, indexes, and interfaces for `CapabilityService` in `code/aiosh-rust/aiosh-core/src/capability_service.rs`.

---

## 2. Scaffold Implementation Details
- Created `code/aiosh-rust/aiosh-core/src/capability_service.rs`.
- Defined `CapabilityService`:
  - Primary store: `capabilities: HashMap<String, Capability>`.
  - Secondary indexes: `by_subject: HashMap<String, HashSet<String>>` and `by_parent: HashMap<String, HashSet<String>>`.
  - Bounded size: `MAX_CAPABILITY_STORE_SIZE = 10 MB`.
- Wired into `code/aiosh-rust/aiosh-core/src/lib.rs` (`pub mod capability_service;` and re-exports).
- Verified compilation with zero warnings.
