# Task Evidence: T-02019 - Capability Model / core service: Documentation (Phase 2, Sub-Epic 2)

## 1. Overview
- **Task ID**: `T-02019`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 2 (core service)
- **Goal**: Document the CapabilityService architecture, invariants CSERV1..CSERV6, and registry custody in `docs/capability_model.md`.

---

## 2. Documentation Updates
- Updated `docs/capability_model.md`:
  - Appended Section 7: "Capability Service & Registry Custody (CSERV1 - CSERV6)"
  - Detailed registry indexing (CSERV1), root issuance control (CSERV2), monotonic attenuation (CSERV3), cascade revocation (CSERV4), safe atomic persistence (CSERV5), and automated pruning (CSERV6).
  - Added service verification instructions for Rust and Python smoke suites.
  - Updated Section 6 with full links to evidence artifacts T-02011 through T-02020.
