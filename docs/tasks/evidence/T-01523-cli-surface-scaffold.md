# T-01523: Filesystem Layout - CLI Surface: Scaffold

## Metadata
- **Task ID:** `T-01523`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface Scaffold (`code/aiosh-rust/aiosh-cli::cmd_fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (3/10) — CLI Surface Scaffold
- **Dependencies:** `T-01522` (CLI Surface Specification)
- **Next Task:** `T-01524` (Filesystem Layout / CLI surface: Implementation)

---

## 1. Scaffold Deliverables

### 1.1 Extended CLI Dispatcher & Interfaces (`cmd_fs_layout`)
In `code/aiosh-rust/aiosh-cli/src/main.rs`, scaffolded the typed function signatures and fail-loud stubs for the remaining 4 subcommands specified in `T-01522`:

1. `cmd_fs_layout_register`:
   - **Signature**: `fn cmd_fs_layout_register(rest: &[String], store_path_opt: Option<&str>, is_json: bool, ctx: &mut Ctx) -> i32`
   - **Syntax**: `aiosh layout register --spec <file_or_json> [--store <path>] [--json]`
   - **Validation**: Verifies `--spec` is present; if omitted, returns exit code `2` (`ARGUMENT_ERROR`).
   - **Scaffold Stub**: Fails loudly with exit code `1` (`NOT_IMPLEMENTED`), emits audit row via `classify_and_emit`.

2. `cmd_fs_layout_set_active`:
   - **Signature**: `fn cmd_fs_layout_set_active(rest: &[String], store_path_opt: Option<&str>, is_json: bool, ctx: &mut Ctx) -> i32`
   - **Syntax**: `aiosh layout set-active <id> [--store <path>] [--json]`
   - **Validation**: Verifies `<id>` positional argument is present; if omitted, returns exit code `2` (`ARGUMENT_ERROR`).
   - **Scaffold Stub**: Fails loudly with exit code `1` (`NOT_IMPLEMENTED`), emits audit row via `classify_and_emit`.

3. `cmd_fs_layout_remove`:
   - **Signature**: `fn cmd_fs_layout_remove(rest: &[String], store_path_opt: Option<&str>, is_json: bool, ctx: &mut Ctx) -> i32`
   - **Syntax**: `aiosh layout remove <id> [--store <path>] [--json]`
   - **Validation**: Verifies `<id>` positional argument is present; if omitted, returns exit code `2` (`ARGUMENT_ERROR`).
   - **Scaffold Stub**: Fails loudly with exit code `1` (`NOT_IMPLEMENTED`), emits audit row via `classify_and_emit`.

4. `cmd_fs_layout_import_fstab`:
   - **Signature**: `fn cmd_fs_layout_import_fstab(rest: &[String], store_path_opt: Option<&str>, is_json: bool, ctx: &mut Ctx) -> i32`
   - **Syntax**: `aiosh layout import-fstab <id> <name> --fstab <file_or_content> [--base <id>] [--store <path>] [--json]`
   - **Validation**: Verifies positional `<id>`, `<name>`, and `--fstab` are present; if omitted, returns exit code `2` (`ARGUMENT_ERROR`).
   - **Scaffold Stub**: Fails loudly with exit code `1` (`NOT_IMPLEMENTED`), emits audit row via `classify_and_emit`.

### 1.2 Store Path Guard & Help Text Updates
- Enforced `--store <path>` validation at the entry of `cmd_fs_layout`: path length must be $\le 1024$ characters and contain zero control characters; violations immediately exit with code `2` (`INVALID_ARGUMENT`).
- Updated top-level `aiosh --help` command overview to include the complete subcommand set `<list|show|validate|check|probe|diff|fstab|register|set-active|remove|import-fstab>`.
- Updated `aiosh layout --help` usage output to document all 10 subcommands and options.

### 1.3 Automated Test Stub Coverage
Extended `test_cmd_fs_layout_flow` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
- Verified store path security guard returns exit code `2` on control-character violation (`\x00`).
- Verified `register` returns exit code `2` on missing `--spec`, and code `1` on scaffolded stub execution.
- Verified `set-active` returns exit code `2` on missing ID, and code `1` on scaffolded stub execution.
- Verified `remove` returns exit code `2` on missing ID, and code `1` on scaffolded stub execution.
- Verified `import-fstab` returns exit code `2` on missing args, and code `1` on scaffolded stub execution.

---

## 2. Compilation & Verification Output

### 2.1 Workspace Check (`cargo check --manifest-path code/aiosh-rust/Cargo.toml`)
```text
Checking aiosh-core v0.1.0
Checking aiosh-sandbox v0.1.0
Checking aiosh-mcp v0.1.0
Checking aiosh-cli v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.95s
```

### 2.2 CLI Test Run (`cargo test ... test_cmd_fs_layout_flow`)
```text
running 1 test
test task_cli_tests::test_cmd_fs_layout_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 1.22s
```

---

## 3. Acceptance Verification
- [x] Project builds and passes `cargo check` with zero errors.
- [x] All 4 new interfaces exist and are wired into `cmd_fs_layout` dispatch.
- [x] Test stubs exercise both argument validation failure paths (code 2) and scaffold stub execution (code 1).
- [x] Consequential actions and errors emit structured audit rows via `classify_and_emit`.
