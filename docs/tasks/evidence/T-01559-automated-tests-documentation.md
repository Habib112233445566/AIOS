# T-01559 — Filesystem Layout automated tests: Documentation

## Metadata
- **Task ID:** `T-01559`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — documented the automated test suite in `docs/filesystem_layout.md`, including test cases A1..A8, execution commands, and honest limitations.
- **Date:** 2026-09-19
- **Depends on:** `T-01558` (Automated Tests Hardening)
- **Feeds:** `T-01560` (Automated Tests Verification & Evidence)
- **Artifacts:** `docs/tasks/evidence/T-01559-automated-tests-documentation.md`

---

## 1. Documentation Updates Summary

1. **`docs/filesystem_layout.md`**:
   - Added Sub-Epic 6: Filesystem Layout Automated Tests (`T-01551`..`T-01560`).
   - Documented the test suite `code/aiosh-cli/tests/test_fs_layout_automated_cases.py` and its 8 core test cases:
     - `A1`: Full lifecycle state machine (`list` -> `register` -> `set-active` -> `show` -> `diff` -> `probe` -> `remove`).
     - `A2`: Built-in layout deletion protection.
     - `A3`: Active layout deletion protection.
     - `A4`: fstab import and 6-field generation cycle.
     - `A5`: Target capacity feasibility probing with boundary checks.
     - `A6`: Differential analysis with destructive change detection.
     - `A7`: Corrupted store tamper resistance and fail-closed file preservation.
     - `A8`: SQLite WAL audit trail verification (ADR-0035).
   - Documented execution commands for standalone and aggregate runner (`tools/test_fs_layout_suites.py`).

2. **Copy-Pasteable Execution Examples**:
   ```bash
   # Run standalone automated cases suite:
   python code/aiosh-cli/tests/test_fs_layout_automated_cases.py

   # Run full aggregate filesystem layout battery (FL1..FL10):
   python tools/test_fs_layout_suites.py
   ```

---

## 2. Honest Limitations Documented

- **Ephemeral Store Precedence:** When running automated test suites, `--store <path>` must be specified; in its absence, tests fall back to in-memory presets which do not persist across separate process invocations.
- **Subprocess Dependency:** Automated test execution relies on compiled binaries (`aiosh.exe` / `aiosh`) existing in `code/aiosh-rust/target/debug/` or `target/debug/`.

---

## 3. Acceptance Confirmation

- [x] Docs updated with working example.
- [x] Limitations are stated, not omitted.
