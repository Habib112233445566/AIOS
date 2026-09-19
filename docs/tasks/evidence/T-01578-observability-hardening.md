# T-01578 — Filesystem Layout observability: Hardening

## Metadata
- **Task ID:** `T-01578`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / observability
- **Status:** Complete — hardened observability and audit log access against timeouts, unhandled errors, and resource leaks.
- **Date:** 2026-09-19
- **Depends on:** `T-01577` (Security Review)
- **Feeds:** `T-01579` (Documentation)
- **Artifacts:** `docs/tasks/evidence/T-01578-observability-hardening.md`, `docs/tasks/evidence/T-01578-hardening.md`

---

## 1. Hardening Measures Implemented

1. **Bounded Subprocess Timeouts & Process Reaping**:
   - MCP stdio communication enforces a 30-second timeout.
   - CLI execution (`aiosh audit tail`, `aiosh layout`) enforces a 60-second cap.
   - On timeout, child processes are terminated (`p.kill()`) and reaped (`p.wait()`) to prevent zombie processes.

2. **Bounded Log Queries & Memory Protection**:
   - `audit tail` requires an explicit count `-n <N>` (defaulting to 60) to avoid loading unbounded SQLite history into memory.
   - Query results are parsed as bounded JSON envelopes.

3. **Resource Leak Prevention**:
   - All tests use context-managed `tempfile.TemporaryDirectory()`, ensuring guaranteed cleanup of ephemeral stores and specs.
   - SQLite WAL file handles are closed immediately after query operations.

4. **Explicit Failure Mode Envelopes (ADR-0035 §F-2)**:
   - Refusals and errors produce structured error envelopes containing `code`, `message`, and `audit_id`.
   - Audit trail records `outcome="error"` or `outcome="refused"` with full diagnostic detail.

---

## 2. Acceptance Confirmation

- [x] Failure modes produce explicit, auditable errors in the standard result envelope.
- [x] No temp file, socket, or child process leaks on error paths.
