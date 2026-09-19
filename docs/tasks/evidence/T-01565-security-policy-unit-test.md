# T-01565 — Filesystem Layout security policy: Unit Test

## Metadata
- **Task ID:** `T-01565`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** Complete — standalone unit testing of the security policy completed with 100% pass rate.
- **Date:** 2026-09-19
- **Depends on:** `T-01564` (Security Policy Implementation)
- **Feeds:** `T-01566` (Security Policy Integration)
- **Artifacts:** `docs/tasks/evidence/T-01565-security-policy-unit-test.md`, `docs/tasks/evidence/T-01565-unit-test.md`

---

## 1. Unit Test Scope & Results

Executed standalone test suite `code/aiosh-cli/tests/test_fs_layout_security_policy.py`:
1. **P1 (Negative - Missing Grant)**:
   - Call to `aios.fs_layout.remove` without `grant_id` -> returns `{"ok": false, "gate": "pep", ...}`.
   - Call to `aios.fs_layout.register` without `grant_id` -> returns `{"ok": false, "gate": "pep", ...}`.
   - Verified that store file is NOT written.
2. **P2 (Negative - Wrong Tool Scope)**:
   - Call with PEP grant scoped to `pentest.*` -> returns `{"ok": false, "gate": "pep", ...}`.
   - Verified that store file is NOT written.
3. **P3 (Negative - Path Scope Violation)**:
   - Call with PEP grant allowing directory A but targeting directory B -> returns `{"ok": false, "gate": "pep", ...}`.
   - Verified error message explicitly names path subject and scope.paths violation.
   - Verified store file is NOT created in the unallowed directory.
4. **P4 (Positive - Valid Scope & Path)**:
   - Call with PEP grant allowing directory and matching `aios.fs_layout.*` -> returns `{"ok": true, ...}`.
   - Verified store file is created on disk.
   - Verified audit record generated with `outcome="ok"`.
5. **P5 (Positive - Read-Only Ungated Access)**:
   - Calls to `get`, `list`, `probe`, `diff`, `fstab` without grant all succeed with `ok: true`.

---

## 2. Test Execution Output

```
=== RUNNING FILESYSTEM LAYOUT SECURITY POLICY TEST SUITE (FL11) ===
PASS: P1 Gated mutation without grant -> refused by PEP gate
PASS: P2 Gated mutation with wrong tool scope -> refused by PEP gate
PASS: P3 Gated mutation with out-of-scope path -> refused by PEP gate
PASS: P4 Gated mutation with valid grant & in-scope paths -> allowed
PASS: P5 Read-only tools (get, list, probe, diff, fstab) executed without grant -> allowed
PASS: All Security Policy criteria P1..P5 passed successfully.
```

---

## 3. Acceptance Confirmation

- [x] All 5 policy cases P1..P5 pass cleanly in standalone execution.
- [x] Fail-closed semantics strictly verified.
