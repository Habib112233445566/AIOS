# T-02623: Secrets Handling CLI Surface Scaffold

- **Task**: `T-02623`
- **Sub-Epic**: Secrets Handling / CLI surface
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Scaffold Deliverable
- Added `aiosh secret` dispatch arm to `main()` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Scaffolded `cmd_secret` routing:
  - `store`: registers secret entry with `--id`, `--name`, `--kind`, `--scope`, `--target`, `--value`.
  - `get`: fetches secret with `--id`, `--scope`, `--target`, `--expose`.
  - `list`: lists vaulted metadata with optional `--kind` and `--scope`.
  - `rotate`: rotates secret with `--id` and `--value`.
  - `revoke`: revokes secret with `--id`.
  - `--help` / usage text.
- Verified zero errors and zero warnings via `cargo check --workspace`.
