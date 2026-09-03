# T-01152 — Base Image Build / Automated Tests: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Automated Tests

## 1. Specification Criteria (T1..T4)
- **T1 (Build Plan Determinism)**:
  - Repeated synthesis of `generate_build_plan` must yield identical JSON structures and deterministic SHA-256 hashes.
- **T2 (Registry Scale & Stress)**:
  - Registering batches of synthetic manifests across various image formats (`Raw`, `Qcow2`, `Iso`, `Tarball`) and ensuring integrity of filtered queries.
- **T3 (Configuration Override Resolution)**:
  - Verifying cascading priority of file configuration, environment variables, and defaults without race conditions.
- **T4 (End-to-End Pipeline Cohesion)**:
  - Full roundtrip from manifest construction to store serialization, reloading from disk, plan generation, and invariant assertion.
