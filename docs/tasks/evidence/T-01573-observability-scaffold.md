# T-01573 — Filesystem Layout observability: Scaffold

## Metadata
- **Task ID:** `T-01573`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / observability
- **Status:** Complete — scaffolded test suite `code/aiosh-cli/tests/test_fs_layout_observability.py` with typed test stubs for O1..O5.
- **Date:** 2026-09-19
- **Depends on:** `T-01572` (Specification)
- **Feeds:** `T-01574` (Implementation)
- **Artifacts:** `docs/tasks/evidence/T-01573-observability-scaffold.md`, `docs/tasks/evidence/T-01573-scaffold.md`

---

## 1. Scaffold Implementation

Created `code/aiosh-cli/tests/test_fs_layout_observability.py`:
- `test_o1_audit_emission_completeness(tmp_dir)`
- `test_o2_audit_correlation_queryability(tmp_dir)`
- `test_o3_outcome_fidelity(tmp_dir)`
- `test_o4_state_inspection_parity(tmp_dir)`
- `test_o5_destructive_mutation_flagging(tmp_dir)`
- Clean import and execution verified via `python code/aiosh-cli/tests/test_fs_layout_observability.py`.

---

## 2. Acceptance Confirmation

- [x] Project builds/imports with zero errors.
- [x] New interfaces exist and are referenced by at least one call site or test stub.
