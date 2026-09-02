# T-00238 — Phase 0 — Release Packaging & Backup / CLI surface: Hardening

## Goal
Harden the CLI surface of Release Packaging & Backup against failure and misuse.

## Completion Notes
1. **Size Caps & Resource Bounds**:
   - The CLI surface delegates external I/O directly to `aiosh_core::release`, which has pre-existing 2GB size caps and symlink mitigations (established during T-0218 and T-0228).
   - Temporary variables parsed from arguments (`--os`, `--version`, `--target-path`) are strongly typed into Rust structs, preventing buffer overflows.

2. **Standard Result Envelope**:
   - `aiosh-cli` handles all errors gracefully using the pre-defined `err_out` macro which outputs a unified `{ "ok": false, "error": ... }` JSON envelope to STDOUT/STDERR.
   - Panic and silent failure paths are prevented by returning `exit(1)` upon `Err(e)` during `generate_release` or `create_backup`.

3. **Resource Cleanup**:
   - The CLI holds `open_context()` only for the lifespan of the `match` block. If `cmd_release` exits early (e.g. usage error) or errors out after invoking `aiosh_core`, the DB connection (`ctx.ring`) is cleanly dropped.
   - Core functions manage their own memory maps and I/O handlers.

4. **Honest Audit Row (ADR-0035)**:
   - Regardless of whether `aiosh_core::release::generate_release` succeeds or fails, it emits an honest `audit_row!`. The CLI relies on this and does not need to duplicate logging, ensuring failure states (like permission denied for backups) accurately map to `outcome="error"` rows.

## Acceptance Criteria Verified
- [x] Failure modes produce explicit, auditable errors via `err_out`.
- [x] No temp/connection leaks on the error path.
