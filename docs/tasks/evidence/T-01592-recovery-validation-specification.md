# T-01592 — Filesystem Layout recovery & validation: Specification

## Metadata
- **Task ID:** `T-01592`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / recovery & validation
- **Status:** Complete — specified recovery & validation contract and test criteria FL14 (R1..R5).
- **Date:** 2026-09-19
- **Depends on:** `T-01591` (Research)
- **Feeds:** `T-01593` (Scaffold)
- **Artifacts:** `docs/tasks/evidence/T-01592-recovery-validation-specification.md`, `docs/tasks/evidence/T-01592-spec.md`

---

## 1. Recovery & Validation Contract Specification

### 1.1 Invariants & Behavior
The Filesystem Layout subsystem provides crash-consistent persistence and fail-closed recovery semantics:

1. **R1: Corruption Refusal & Containment**:
   - When a store file contains invalid JSON, truncated bytes, or schema violations (`deny_unknown_fields`), loading operations fail closed with `LOAD_STORE_FAILED`.
   - Under no circumstances is the corrupted file rewritten, truncated, or deleted by the loader.
2. **R2: Fallback to Canonical Presets**:
   - In the absence of a persisted store file, queries return built-in presets (`aios-uefi-standard-v1` and `aios-container-minimal-v1`) from memory.
3. **R3: Recovery via Valid Replacement**:
   - Restoring a valid store file (e.g. from backup) allows immediate resumption of operations without service restarts.
4. **R4: Atomic Write Crash Consistency**:
   - State mutations stage writes in temporary files (`.tmp.<pid>`) and call `fsync` before atomic renaming.
   - If an error occurs during validation or staging, the existing target store file remains unaltered.
5. **R5: Audit Trail Continuity**:
   - Both corrupted load attempts and subsequent recoveries emit corresponding audit records into SQLite WAL `audit.db` with appropriate outcomes (`error` vs `ok`).

---

## 2. Automated Test Criteria (FL14)

The test suite `code/aiosh-cli/tests/test_fs_layout_recovery_validation.py` exercises:
- `test_r1_corruption_refusal_containment`: Truncated/corrupted store is refused; file contents remain untouched.
- `test_r2_fallback_to_canonical_presets`: Absent store returns in-memory presets cleanly.
- `test_r3_recovery_via_valid_replacement`: Replacing corrupted file with valid backup restores service.
- `test_r4_atomic_write_crash_consistency`: Failed mutation leaves existing store completely intact.
- `test_r5_audit_trail_continuity`: Confirms audit events logged for failed and recovered operations.

---

## 3. Acceptance Confirmation

- [x] Spec covers happy path, failure path, and audit effects.
- [x] Spec is reviewable without reading the implementation.
