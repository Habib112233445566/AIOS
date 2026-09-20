# Task Evidence: T-02009 - Capability Model / data model: Documentation (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02009`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Document the Capability Model data model in `docs/capability_model.md` for operators and autonomous agents.

---

## 2. Documentation Summary
Created `docs/capability_model.md`:
- **Architecture**: Zero ambient authority principles, unforgeable tokens, monotonic attenuation, and transitive revocation.
- **Invariants**: Complete descriptions and mathematical/logical constraints for `CAP1..CAP6`.
- **Data Models**: Rust definitions and JSON schema for `CapabilityRight`, `CapabilityScope`, `CapabilityConstraints`, and `Capability`.
- **Operational Procedures**: Attenuation rules, lifecycle verification, and execution commands for Rust unit tests and Python smoke suites.
