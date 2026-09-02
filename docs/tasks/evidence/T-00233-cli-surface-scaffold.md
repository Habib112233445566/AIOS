# T-00233 — Phase 0 — Release Packaging & Backup / CLI surface: Scaffold

## Goal
Create the module skeleton and interfaces for the CLI surface of Release Packaging & Backup.

## Completion Notes
1. **Module Scaffolding (`aiosh-cli/src/main.rs`)**:
   - Added `cmd_release` and `cmd_backup` stubs returning `i32` (standard CLI exit code).
   - Wired the top-level CLI router `match args.first()` to direct `"release"` to `cmd_release` and `"backup"` to `cmd_backup`.
   - Updated the `--help` block to declare the new `aiosh release` and `aiosh backup` commands.

2. **Fails Loudly**:
   - Stubs use `unimplemented!("Scaffold: cmd_release pending implementation (T-234)")` to panic reliably if invoked prematurely.

3. **Compilation Fixes**:
   - Cleaned up a type mismatch in `aiosh-core/src/release.rs` where `c_flags: CFlags { c1: 0, ... }` was supplied instead of `c1: false`. (This was my own compilation error from a previous task, resolved now).
   - Ignored unrelated Windows-specific `libc` build failures (`syscall` missing on Windows) which are well documented in the project's OS parity constraints.

## Acceptance Criteria Verified
- [x] New interfaces exist and are referenced by at least one call site (the main loop router).
- [x] Project imports/builds cleanly aside from the pre-existing Windows OS `libc` conditional compilation issues.
