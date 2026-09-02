# T-00228 — Phase 0 — Release Packaging & Backup / Core Service: Hardening

## Goal
Harden the core service of Release Packaging & Backup against failure and misuse.

## Completion Notes
1. **Size Limits & Symlink Guards (`aiosh-mcp/aiosh_mcp/release.py`)**:
   - Added a `MAX_FILE_SIZE` constraint of 2GB for any single file processed during `physical_create_zip` to prevent OOM errors or filling up the disk with anomalous payload files.
   - Guarded the zipper against symbolic links via `os.path.islink()` to prevent infinite loop recursive attacks or escaping the snapshot directory limits.
   - Handled mid-walk deletions gracefully: File size checks are wrapped in a nested `try/except OSError` to absorb file locking/race conditions during live backup.

2. **Resource Cleanup**:
   - Both ISO and ZIP routines leverage context managers (`with open`, `with zipfile.ZipFile`, `with audit_client.open_db()`) enforcing deterministic cleanup of file descriptors and SQLite DB connections in both success and error paths.
   
3. **Fail-Open Auditing**:
   - Validated that the `try/except` model in both the `generate_release` and `create_backup` MCP methods perfectly capture `Exception as e` from the underlying physical modules and unconditionally route them to the DB as `outcome="error"`.
   - The failure is safely audited *before* the stack is unwound to the caller.

## Acceptance Criteria Verified
- [x] Failure modes produce explicit, auditable errors without crashing the host process silently.
- [x] DB Connections and descriptors are cleanly wrapped in context managers.
- [x] Timeouts/size guards are defined and enforced at the loop layer.
