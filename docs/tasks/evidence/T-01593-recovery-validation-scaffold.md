# T-01593 — Filesystem Layout recovery & validation: Scaffold

## Metadata
- **Task ID:** `T-01593`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / recovery & validation
- **Status:** Complete — scaffolded test suite `code/aiosh-cli/tests/test_fs_layout_recovery_validation.py` with typed stubs for R1..R5.
- **Date:** 2026-09-19
- **Depends on:** `T-01592` (Specification)
- **Feeds:** `T-01594` (Implementation)
- **Artifacts:** `docs/tasks/evidence/T-01593-recovery-validation-scaffold.md`, `docs/tasks/evidence/T-01593-scaffold.md`

---

## 1. Scaffold Implementation

Created `code/aiosh-cli/tests/test_fs_layout_recovery_validation.py`:
- `test_r1_corruption_refusal_containment(tmp_dir)`
- `test_r2_fallback_to_canonical_presets(tmp_dir)`
- `test_r3_recovery_via_valid_replacement(tmp_dir)`
- `test_r4_atomic_write_crash_consistency(tmp_dir)`
- `test_r5_audit_trail_continuity(tmp_dir)`
- Clean import and execution verified via `python code/aiosh-cli/tests/test_fs_layout_recovery_validation.py`.

---

## 2. Acceptance Confirmation

- [x] Project builds/imports with zero errors.
- [x] New interfaces exist and are referenced by at least one call site or test stub.
