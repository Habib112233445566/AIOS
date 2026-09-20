# Task Evidence: T-02006 - Capability Model / data model: Integration (Phase 2, Sub-Epic 1)

## 1. Overview
- **Task ID**: `T-02006`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 1 (data model)
- **Goal**: Integrate the Capability Model data model with surrounding system crates, re-exporting in `aiosh-core::lib`, and providing cross-substrate verification in `code/aiosh-mcp/tests/test_capability_smoke.py`.

---

## 2. Integration Details
- **Core Crate Integration**:
  - Re-exported `Capability`, `CapabilityRight`, `CapabilityScope`, `CapabilityConstraints`, `CapabilityError`, and associated constants in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- **Cross-Substrate Verification**:
  - Created `code/aiosh-mcp/tests/test_capability_smoke.py` validating:
    - JSON serialization and deserialization parity.
    - Scope matching for filesystem and network wildcards.
    - Monotonic attenuation rules (`CAP3`) preventing privilege escalation.
    - Quota and revocation tracking (`CAP4`, `CAP5`).
- **Smoke Suite Execution**:
  - Command: `python code/aiosh-mcp/tests/test_capability_smoke.py`
  - Result: **PASS** (100% assertions verified).
