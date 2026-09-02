# T-00218 — Phase 0 — Release Packaging & Backup / Data Model: Hardening

## Goal
Harden the data model of Release Packaging & Backup against failure and misuse.

## Completion Notes
1. **Error Standard Envelope (`aiosh_mcp/server.py`)**:
   - The MCP tool endpoints (`aios_release_generate` and `aios_backup_create`) are wrapped in `try/except Exception as exc:` blocks.
   - Any runtime failure, either in dispatch gate validation or pure Python generation, correctly emits a `"error"` outcome row to the audit ring, and returns `{"ok": False, "error": str(exc), "audit_id": row.id}`. This prevents silent failures or unrecorded errors (meeting ADR-0035 F-2).
2. **Resource Cleanup (`aiosh_mcp/release.py`)**:
   - The `audit_client.open_db()` connection relies on the context manager (`with audit_client.open_db() as conn:`) ensuring the WAL db handles commit/rollback correctly on completion or exception.
3. **No External I/O**:
   - At the data model layer, path strings and hashes are generated purely in-memory. Timeouts or size caps for file sizes and process execution will be implemented in the later core backend phases (Task T-00222 "core logic"). No blocking external processes currently exist here.

## Acceptance Criteria Verified
- [x] Failure modes produce explicit, auditable errors (guaranteed via `try/except` -> `dispatch_mod.commit` flow).
- [x] No temp/connection leaks on the error path (handled by context managers).
