# T-01594 — Filesystem Layout recovery & validation: Implementation

## Metadata
- **Task ID:** `T-01594`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / recovery & validation
- **Status:** Complete — implemented recovery & validation test suite `code/aiosh-cli/tests/test_fs_layout_recovery_validation.py` exercising R1..R5.
- **Date:** 2026-09-19
- **Depends on:** `T-01593` (Scaffold)
- **Feeds:** `T-01595` (Unit Test)
- **Artifacts:** `docs/tasks/evidence/T-01594-recovery-validation-implementation.md`, `docs/tasks/evidence/T-01594-implementation.md`

---

## 1. Implementation Details

Implemented the recovery & validation test cases in `code/aiosh-cli/tests/test_fs_layout_recovery_validation.py`:
- **R1 (Corruption Refusal & Containment)**:
  - Asserts that corrupted/truncated store files return `LOAD_STORE_FAILED` (CLI) and fail closed on MCP.
  - Confirms file hash is unchanged (tamper-resistant, no overwrites).
- **R2 (Fallback to Canonical Presets)**:
  - Asserts that queries executed without an explicit store file fall back cleanly to in-memory presets (`aios-uefi-standard-v1`, `aios-container-minimal-v1`) on both CLI and MCP.
- **R3 (Recovery via Valid Replacement)**:
  - Corrupts store, confirms failure, restores valid layout store, and confirms immediate operational recovery on both CLI and MCP without service restarts.
- **R4 (Atomic Write Crash Consistency)**:
  - Attempts an invalid mutation (mode 0) against a valid store; confirms existing store file remains intact with original hash and zero leaked staging files (`.tmp.*`).
- **R5 (Audit Trail Continuity)**:
  - Asserts that failed operations log `outcome="error"` and subsequent recovery operations log `outcome="ok"`/`"success"` in SQLite WAL `audit.db`.

---

## 2. Test Execution Output

```
=== RUNNING FILESYSTEM LAYOUT RECOVERY & VALIDATION TEST SUITE (FL14) ===
PASS: R1 Corruption refusal & containment verified (tamper-resistant, fail-closed)
PASS: R2 Fallback to canonical presets verified on CLI and MCP
PASS: R3 Recovery via valid replacement restores full functionality
PASS: R4 Atomic write crash consistency verified (zero staging leaks, unmodified store)
PASS: R5 Audit trail continuity verified for both failure and recovery operations
PASS: All Recovery & Validation criteria R1..R5 passed successfully.
```

---

## 3. Acceptance Confirmation

- [x] Targeted test passes.
- [x] No regression in existing smoke suites for touched modules.
