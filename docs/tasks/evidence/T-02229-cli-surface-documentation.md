# Task Evidence: T-02229 (Grant Lifecycle / CLI surface: Documentation)

## 1. Metadata
- **Task ID:** `T-02229`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle CLI Surface Documentation (`docs/pep_decision_engine.md`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic: Grant Lifecycle (3/10) — CLI Surface Documentation

---

## 2. Documentation Summary

Updated authoritative system documentation in `docs/pep_decision_engine.md` with **Section 17: Grant Lifecycle CLI Surface Reference**:
1. **Subcommands Catalog**: Documented `issue`, `attenuate`, `list`, `inspect`, `validate`, `revoke`, and `sweep` including mandatory flags and options.
2. **Copy-Pasteable Workflows**: Provided practical invocations for root issuance, attenuation, validation, recursive cascade revocation, and expiration sweeping.
3. **Constraints & Known Limitations**: Documented mandatory delegation rights, strict subset monotonic attenuation, depth decrementing, path hygiene constraints, and JSON envelope schemas.
4. **Task Evidence Traceability**: Cross-linked evidence documents `T-02221` through `T-02230`.

---

## 3. Acceptance Confirmation
- [x] Documentation updated with working examples.
- [x] Constraints and limitations documented.
- [x] Task evidence links included in specification.
