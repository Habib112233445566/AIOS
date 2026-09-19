# T-01563 — Filesystem Layout security policy: Scaffold

## Metadata
- **Task ID:** `T-01563`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** Complete — scaffolded test suite module `code/aiosh-cli/tests/test_fs_layout_security_policy.py` with typed test stubs P1..P5.
- **Date:** 2026-09-19
- **Depends on:** `T-01562` (Security Policy Specification)
- **Feeds:** `T-01564` (Security Policy Implementation)
- **Artifacts:** `docs/tasks/evidence/T-01563-security-policy-scaffold.md`, `docs/tasks/evidence/T-01563-scaffold.md`

---

## 1. Scaffolded Module Structure

- **Target File:** `code/aiosh-cli/tests/test_fs_layout_security_policy.py`
- **Signatures Scaffolded:**
  - `get_binary_path() -> str`
  - `test_p1_mutation_without_grant(tmp_dir: Path) -> None`
  - `test_p2_mutation_wrong_tool_scope(tmp_dir: Path) -> None`
  - `test_p3_mutation_out_of_scope_path(tmp_dir: Path) -> None`
  - `test_p4_mutation_valid_grant(tmp_dir: Path) -> None`
  - `test_p5_readonly_ungated(tmp_dir: Path) -> None`
  - `main() -> int`

---

## 2. Verification

- Verified clean import:
  `python -c "import sys; sys.path.insert(0, 'code/aiosh-cli/tests'); import test_fs_layout_security_policy; print('Import OK')"` -> **Import OK**.
- Project builds cleanly with 0 errors.

---

## 3. Acceptance Confirmation

- [x] Project builds/imports with zero errors.
- [x] New interfaces exist and are referenced by at least one call site or test stub.
