# T-01556 — Filesystem Layout automated tests: Integration

## Metadata
- **Task ID:** `T-01556`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — integrated `code/aiosh-cli/tests/test_fs_layout_automated_cases.py` into `tools/test_fs_layout_suites.py` as criterion **FL10**; verified end-to-end execution.
- **Date:** 2026-09-19
- **Depends on:** `T-01555` (Automated Tests Unit Test)
- **Feeds:** `T-01557` (Automated Tests Security Review)
- **Artifacts:** `docs/tasks/evidence/T-01556-automated-tests-integration.md`, `docs/tasks/evidence/T-01556-integration.md`

---

## 1. Integration Scope

- **Registered Criterion:** Criterion **FL10** in `tools/test_fs_layout_suites.py`.
- **Target Harness:** `code/aiosh-cli/tests/test_fs_layout_automated_cases.py`.
- **Coverage:** End-to-end lifecycle, protection of built-ins and active layouts, fstab import/export cycle, capacity feasibility probing, differential destructive change detection, corrupt store tamper resistance, and SQLite WAL audit logging.
- **Cross-Substrate Parity:** Evaluates the real compiled Rust binaries (`aiosh.exe` / `aiosh`) over subprocess boundaries against isolated SQLite WAL databases and JSON stores.

---

## 2. Updated Runner Definition

In `tools/test_fs_layout_suites.py`:
```python
def test_fl10_automated_cases() -> bool:
    return _run_python_script(
        ROOT / "code" / "aiosh-cli" / "tests" / "test_fs_layout_automated_cases.py",
        "FL10",
        "filesystem layout automated lifecycle & edge cases (A1..A8: state machine, "
        "protection, fstab, probe, diff, corruption, audit)",
    )
```

Added to `suites` array:
```python
    suites = [
        ("FL1", test_fl1_data_model),
        ("FL2", test_fl2_core_service),
        ("FL3", test_fl3_cli_surface),
        ("FL4", test_fl4_cli_audit_security),
        ("FL5", test_fl5_in_tree_unit_tests),
        ("FL6", test_fl6_cross_surface_integration),
        ("FL7", test_fl7_cli_hardening),
        ("FL8", test_fl8_mcp_contract),
        ("FL9", test_fl9_configuration_contract),
        ("FL10", test_fl10_automated_cases),
    ]
```

---

## 3. Acceptance Confirmation

- [x] Feature reachable through its production surface.
- [x] Integration smoke passes end-to-end.
