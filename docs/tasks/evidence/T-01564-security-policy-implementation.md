# T-01564 — Filesystem Layout security policy: Implementation

## Metadata
- **Task ID:** `T-01564`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** Complete — security policy test cases P1..P5 implemented and verified against `aiosh-mcp` and `aiosh` PEP gate.
- **Date:** 2026-09-19
- **Depends on:** `T-01563` (Security Policy Scaffold)
- **Feeds:** `T-01565` (Security Policy Unit Test)
- **Artifacts:** `docs/tasks/evidence/T-01564-security-policy-implementation.md`, `docs/tasks/evidence/T-01564-implementation.md`

---

## 1. Implementation Summary

Implemented the test suite in `code/aiosh-cli/tests/test_fs_layout_security_policy.py`:
- **P1**: Gated mutation without grant (`aios.fs_layout.remove`, `aios.fs_layout.register`) -> refused by PEP gate (`gate == "pep"`), store file not created.
- **P2**: Gated mutation with wrong tool scope (grant scoped to `pentest.*`) -> refused by PEP gate, store file not created.
- **P3**: Gated mutation with out-of-scope path (grant allows `allowed_dir`, mutation targets `outside_dir`) -> refused by PEP gate with `scope.paths` and `path subject` in error.
- **P4**: Gated mutation with valid grant & in-scope paths -> allowed (`ok == True`), store written, audit row verified with `outcome="ok"`.
- **P5**: Read-only tools (`get`, `list`, `probe`, `diff`, `fstab`) without grant -> allowed (`ok == True`).

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

- [x] P1..P5 implemented and executed successfully against live binaries.
- [x] Fail-closed behavior validated for ungranted, wrong-scoped, and out-of-scope operations.
- [x] Read-only operations remain accessible without authorization grants.
