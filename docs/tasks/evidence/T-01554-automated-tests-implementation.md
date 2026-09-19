# T-01554 — Filesystem Layout automated tests: Implementation

## Metadata
- **Task ID:** `T-01554`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — implemented end-to-end automated test suite `code/aiosh-cli/tests/test_fs_layout_automated_cases.py` exercising cases A1..A8.
- **Date:** 2026-09-19
- **Depends on:** `T-01553` (Automated Tests Scaffold)
- **Feeds:** `T-01555` (Automated Tests Unit Test)
- **Artifacts:** `docs/tasks/evidence/T-01554-automated-tests-implementation.md`, `docs/tasks/evidence/T-01554-implementation.md`

---

## 1. Implementation Details

Delivered `code/aiosh-cli/tests/test_fs_layout_automated_cases.py` with 8 comprehensive automated test cases:
1. `test_a1_full_lifecycle`: Exercises state machine flow (`list` -> `register` -> `set-active` -> `show` -> `diff` -> `probe` -> restore active -> `remove`).
2. `test_a2_builtin_protection`: Asserts deletion refusal for built-in presets (`aios-container-minimal-v1`).
3. `test_a3_active_protection`: Asserts deletion refusal for active layout (`aios-uefi-standard-v1`).
4. `test_a4_fstab_cycle`: Imports real fstab, verifies synthesized layout, and exports 6-field `fstab(5)` format.
5. `test_a5_probe_feasibility`: Tests undersized (< min bytes), standard minimum, tight capacity (<10% slack warning), and generous capacity.
6. `test_a6_diff_destructive`: Verifies differential analysis flags destructive alterations (`destructive: true`) when partitions are shrunk.
7. `test_a7_corrupt_store_recovery`: Verifies tamper resistance against corrupted store files without data loss.
8. `test_a8_audit_emission`: Verifies SQLite WAL hash-chained audit row emission to `audit.db`.

---

## 2. Verification Run

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

- [x] Targeted test passes.
- [x] No regression in existing smoke suites for touched modules.
