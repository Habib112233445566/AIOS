# T-00303 — Release Packaging & Backup: Recovery & Validation Scaffold

## Scaffold Implementation
We scaffolded the recovery and validation interfaces in `aiosh-core/src/release.rs` to prepare for physical implementation.

The following interfaces were defined:
- `validate_release(_artifact_path: &str, _expected_hash: &str) -> Result<(), String>`
- `validate_backup(_backup_path: &str) -> Result<(), String>`
- `restore_backup(_ctx: &mut ReleaseCtx, _backup_path: &str, _target_dir: &str) -> Result<(), String>`

All interfaces return a standard Rust `Result` and are currently hardcoded to fail loudly with `Err("Not implemented")` to prevent silent fallthrough during the upcoming implementation task.

## Validation
- Ran `cargo check` on the repository.
- The project successfully compiles with 0 errors, confirming the interface scaffolding is syntactically sound and correctly exported.
- The task is structurally complete.
