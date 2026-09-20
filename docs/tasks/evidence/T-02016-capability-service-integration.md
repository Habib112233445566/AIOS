# Task Evidence: T-02016 - Capability Model / core service: Integration (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02016`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Integrate `CapabilityService` with surrounding system crates, re-exporting in `aiosh-core::lib`, and providing cross-substrate verification in `code/aiosh-mcp/tests/test_capability_service_smoke.py`.

---

## 2. Integration Details
- **Core Crate Integration**:
  - Re-exported `CapabilityService`, `CSERV_IO_ERROR`, `CSERV_NOT_FOUND`, `CSERV_VALIDATION_ERROR`, and `MAX_CAPABILITY_STORE_SIZE` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- **Cross-Substrate Verification**:
  - Authored `code/aiosh-mcp/tests/test_capability_service_smoke.py` validating:
    - Root capability issuance and subject indexing (`CSERV2`).
    - Monotonic child attenuation and parent-child lineage tracking (`CSERV3`).
    - Transitive cascade revocation across multi-level delegation trees (`CSERV4`).
    - Safe disk persistence and restoration roundtrip (`CSERV5`).
- **Smoke Suite Execution**:
  - Command: `python code/aiosh-mcp/tests/test_capability_service_smoke.py`
  - Result: **PASS** (100% assertions verified).
