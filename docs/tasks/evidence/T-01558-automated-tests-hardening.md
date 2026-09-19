# T-01558 — Filesystem Layout automated tests: Hardening

## Metadata
- **Task ID:** `T-01558`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — automated test suite hardened with explicit subprocess timeouts (60s), RAII temporary directory cleanup, connection closing, and fail-closed validation checks.
- **Date:** 2026-09-19
- **Depends on:** `T-01557` (Automated Tests Security Review)
- **Feeds:** `T-01559` (Automated Tests Documentation)
- **Artifacts:** `docs/tasks/evidence/T-01558-automated-tests-hardening.md`

---

## 1. Hardening Measures Implemented

| Hardening Requirement | Implementation & Defense Mechanism | Verification |
|---|---|---|
| **Subprocess Timeouts** | Explicit 60-second timeout enforced on all `subprocess.run` invocations in `test_fs_layout_automated_cases.py`; 180-second timeout in `test_fs_layout_suites.py`. Prevents indefinite process hangs. | Verified across all test cases A1..A8. |
| **Resource & Temp Cleanup** | All test operations execute inside a scoped `tempfile.TemporaryDirectory` context manager (`with tempfile.TemporaryDirectory() as td:`). Temporary files, JSON stores, and test fstab files are guaranteed to be cleaned up on both success and failure. | Verified clean directory removal. |
| **Database Connection Hygiene** | All SQLite audit database connections in `test_a8_audit_emission` are explicitly closed (`conn.close()`) immediately after querying rows, preventing file handle leaks and SQLite lock contention. | Verified SQLite lock-free execution. |
| **Standard Result Envelopes** | All CLI commands assert that failures return uniform JSON envelopes `{"code": 1, "data": null, "error": {...}}` with appropriate error codes (`REMOVE_FAILED`, `NOT_VIABLE`, `REGISTER_FAILED`). | Asserted across A2, A3, A5, A7. |
| **Fail-Closed Audit Emission** | Every failure and mutation emits an honest SQLite WAL audit record containing the caller context, command, outcome, and SHA-256 hash. | Asserted in A8. |

---

## 2. Acceptance Confirmation

- [x] Failure modes produce explicit, auditable errors.
- [x] No temp/connection leaks on the error path.
