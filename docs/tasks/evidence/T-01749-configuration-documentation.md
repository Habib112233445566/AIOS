# T-01749: Hardware Detection — Configuration Documentation

## Metadata
- **Task ID**: `T-01749`
- **Sub-Epic**: Sub-Epic 5: Hardware Detection Configuration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Documentation Scope
Authored comprehensive documentation for Sub-Epic 5 (Configuration) in Section 11 of `docs/hardware_detection.md`.

---

## 2. Documented Topics
- **Architecture & Structure**: Overview of `HardwareConfig` in `aiosh-core::hardware_config`.
- **Data Contract**: Rust struct definition, fields, types, and defaults.
- **Invariants HCFG1..HCFG5**: Formal definitions of path hygiene, class filtering, resource bounds, timeout bounds, and serialization fallback.
- **Environment Variables**: Table describing `AIOSH_HARDWARE_CONFIG`, `AIOSH_HARDWARE_SYSFS`, `AIOSH_HARDWARE_PROCFS`, `AIOSH_HARDWARE_STORE`, `AIOSH_HARDWARE_INCLUDE_ATTRS`, and `AIOSH_HARDWARE_TIMEOUT_SECS`.
- **Evidence References**: Full index of tasks `T-01741` through `T-01750`.
