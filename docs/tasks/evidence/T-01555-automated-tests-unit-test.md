# T-01555 — Filesystem Layout automated tests: Unit Test

## Metadata
- **Task ID:** `T-01555`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — unit test verification completed for `test_fs_layout_automated_cases.py` asserting valid, invalid, boundary, and corruption recovery cases.
- **Date:** 2026-09-19
- **Depends on:** `T-01554` (Automated Tests Implementation)
- **Feeds:** `T-01556` (Automated Tests Integration)
- **Artifacts:** `docs/tasks/evidence/T-01555-automated-tests-unit-test.md`, `docs/tasks/evidence/T-01555-unit-test.md`

---

## 1. Test Coverage & Negative Assertions

The automated test suite `code/aiosh-cli/tests/test_fs_layout_automated_cases.py` rigorously covers:
1. **Valid Happy Paths**:
   - `test_a1_full_lifecycle`: Ingestion, state mutation, active selection, diffing, and deletion.
   - `test_a4_fstab_cycle`: fstab parsing and 6-field generation.
2. **Negative Cases & Protection Invariants**:
   - `test_a2_builtin_protection`: Negative assertion that built-in layouts (`aios-container-minimal-v1`) cannot be deleted, returning exit code 1.
   - `test_a3_active_protection`: Negative assertion that the active layout (`aios-uefi-standard-v1`) cannot be deleted, returning exit code 1.
   - `test_a7_corrupt_store_recovery`: Negative assertion that a syntactically corrupted store file causes commands to fail closed without overwriting or destroying the on-disk file.
3. **Boundary Values**:
   - `test_a5_probe_feasibility`: Asserts three distinct boundary zones:
     - Boundary 1: Undersized storage (< min bytes) -> exit code 1, `is_viable: false`.
     - Boundary 2: Standard minimum storage (64 GiB) -> exit code 0, `is_viable: true`, `errors: []`.
     - Boundary 3: Tight storage (< 10% slack) -> exit code 0, `warnings` contains 10% headroom warning.
     - Boundary 4: Generous storage (128 GiB) -> exit code 0, `warnings: []`.
   - `test_a6_diff_destructive`: Asserts that partition shrinking triggers `destructive: true` in diff summary.
4. **Audit Invariants (ADR-0035)**:
   - `test_a8_audit_emission`: Asserts `tool="fs_layout"`, `command="validate"`, `outcome="success"`, and SHA-256 hash length == 64.

---

## 2. Standalone Execution

Executed `python code/aiosh-cli/tests/test_fs_layout_automated_cases.py`:
- Exit Code: `0`
- Results:
  ```
  Running FL10 Automated Cases...
    [+] A1: Full lifecycle state machine PASS
    [+] A2: Built-in protection PASS
    [+] A3: Active layout protection PASS
    [+] A4: fstab import & generation roundtrip PASS
    [+] A5: Probe capacity feasibility PASS
    [+] A6: Differential analysis destructive detection PASS
    [+] A7: Corrupt store tamper resistance PASS
    [+] A8: SQLite WAL audit trail emission PASS
  All FL10 automated cases passed successfully!
  ```

---

## 3. Acceptance Confirmation

- [x] New test file runs standalone and passes.
- [x] Negative cases are asserted, not just happy path.
