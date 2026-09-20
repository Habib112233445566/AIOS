# Task Evidence: T-01929 - System Update / CLI surface: Documentation

- **Task**: `T-01929`
- **Sub-Epic**: `Sub-Epic 3: Operator CLI & Control Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Authored comprehensive documentation for the Operator CLI Subsystem in `docs/system_update.md` (Section 6):
- Documented command grammar (`aiosh update`, alias `aiosh upd`).
- Documented subcommands: `status`, `slots`, `check`, `apply`, `confirm`, `rollback`.
- Documented flags and options: `--state-dir`, `--staging-dir`, `--version`, `--slot`, `--json`, `--help`.
- Documented exit codes (0 = Success, 1 = Domain Error, 2 = CLI / Path Error).
- Documented JSON output envelope schema with error object definitions.
- Defined CLI invariants `UCLI1..UCLI6` (deterministic routing, path hygiene enforcement, structured serialization, audit trail integrity, hermetic isolation, and terminal safety).

## Verification
- Verified documentation syntax and formatting in `docs/system_update.md`.
