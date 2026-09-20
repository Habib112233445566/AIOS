# Evidence: T-02061 - research

## Task Overview
- **Task ID**: `T-02061`
- **Sub-Epic**: Sub-Epic 7: Security Policy (`T-02061`..`T-02070`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Action**: Research capability security policy principles, prior art, facts vs assumptions, and invariant design (`CAPSEC1..6`).

## Research Summary
- Authored `docs/tasks/evidence/T-02061-security-policy-research.md`.
- Analyzed foundational literature (Dennis & Van Horn, Saltzer & Schroeder, Miller et al., seL4).
- Established policy invariants `CAPSEC1..6` covering default-deny, attenuation depth limits, sensitive resource restrictions, subject disallowed rights, temporal validity ceilings, and deterministic verdict reporting.
- Identified integration points with `CapabilityService`.
