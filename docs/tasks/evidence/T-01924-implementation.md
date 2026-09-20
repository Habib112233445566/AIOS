# Task Evidence: T-01924 - System Update / CLI surface: Implementation

- **Task**: `T-01924`
- **Sub-Epic**: `Sub-Epic 3: Operator CLI & Control Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Work
Implemented operator control surface subcommands and routines for System Update in `code/aiosh-rust/aiosh-cli/src/main.rs`:
- `status`: Displays current slot, candidate slot, state, versions, failure counts.
- `slots`: Shows detailed slot health for Slot A and Slot B (state, version, priority, boot attempts, successful boots).
- `check`: Loads manifest JSON, validates schema and version semantics, verifies signature / integrity requirements via `service.check_manifest(manifest)`.
- `apply`: Validates staged payload and marks candidate slot as active for next reboot via `service.apply_update()`.
- `confirm`: Confirms boot stability on active slot and increments successful boot counter via `service.confirm_boot()`.
- `rollback`: Reverts active boot slot to stable alternate slot and flags bad slot via `service.rollback()`.
- Deterministic exit codes: 0 for success, 1 for domain/operational failures, 2 for syntax/flag/path hygiene errors.
- Structured JSON envelopes with `{ "code": i32, "data": Any, "error": Any }`.
- Integrated audit logging with `classify_and_emit` into the SQLite WAL audit ring.

## Verification
- Code successfully builds and passes all check validations via `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli`.
