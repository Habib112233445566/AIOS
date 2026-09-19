# T-01553 — Filesystem Layout automated tests: Scaffold

## Metadata
- **Task ID:** `T-01553`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — scaffolded test suite module `code/aiosh-cli/tests/test_fs_layout_automated_cases.py` with typed test stubs A1..A8.
- **Date:** 2026-09-19
- **Depends on:** `T-01552` (Automated Tests Specification)
- **Feeds:** `T-01554` (Automated Tests Implementation)
- **Artifacts:** `docs/tasks/evidence/T-01553-automated-tests-scaffold.md`, `docs/tasks/evidence/T-01553-scaffold.md`

---

## 1. Scaffolded Module Structure

- **Target File:** `code/aiosh-cli/tests/test_fs_layout_automated_cases.py`
- **Signatures Scaffolded:**
  - `get_binary_path() -> str`
  - `run_aiosh(args: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess`
  - `test_a1_full_lifecycle(binary: str, tmp_dir: Path) -> None`
  - `test_a2_builtin_protection(binary: str, tmp_dir: Path) -> None`
  - `test_a3_active_protection(binary: str, tmp_dir: Path) -> None`
  - `test_a4_fstab_cycle(binary: str, tmp_dir: Path) -> None`
  - `test_a5_probe_feasibility(binary: str, tmp_dir: Path) -> None`
  - `test_a6_diff_destructive(binary: str, tmp_dir: Path) -> None`
  - `test_a7_corrupt_store_recovery(binary: str, tmp_dir: Path) -> None`
  - `test_a8_audit_emission(binary: str, tmp_dir: Path) -> None`
  - `main() -> int`

---

## 2. Verification

- Verified clean import:
  `python -c "import sys; sys.path.insert(0, 'code/aiosh-cli/tests'); import test_fs_layout_automated_cases; print('Import OK')"` -> **Import OK**.
- Project builds cleanly with 0 errors.

---

## 3. Acceptance Confirmation

- [x] Project builds/imports with zero errors.
- [x] New interfaces exist and are referenced by at least one call site or test stub.
