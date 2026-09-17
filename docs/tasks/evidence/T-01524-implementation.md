# T-01524: Filesystem Layout - CLI Surface: Implementation

## Metadata
- **Task ID:** `T-01524`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface (`code/aiosh-rust/aiosh-cli::cmd_fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-17
- **Milestone:** Sub-Epic: Filesystem Layout (4/10) — CLI Surface Implementation
- **Dependencies:** `T-01523` (CLI Surface Scaffold)
- **Next Task:** `T-01525` (Filesystem Layout / CLI surface: Unit Test)

---

## 1. Summary of Changes

In accordance with specification `T-01522-spec.md`, the four mutating subcommands scaffolded in
`T-01523` were implemented in `code/aiosh-rust/aiosh-cli/src/main.rs`, replacing the fail-loud
`NOT_IMPLEMENTED` stubs with real store-backed behavior:

1. **`cmd_fs_layout_register(rest, store_path_opt, is_json, ctx)`** — `aiosh layout register --spec <file_or_json> [--store <path>] [--json]`
   - Resolves `--spec` from either an on-disk file or an inline JSON string.
   - Enforces a 10 MiB file ceiling before reading, then validates `FL1..FL5` before insertion.
   - Registers into the store via `FilesystemLayoutStore::register_layout` (duplicate IDs rejected with exit code `1`).
   - Persists atomically with `FilesystemLayoutService::save_to_path`.

2. **`cmd_fs_layout_set_active(rest, store_path_opt, is_json, ctx)`** — `aiosh layout set-active <id> [--store <path>] [--json]`
   - Parses the positional `<id>` and rejects omission with exit code `2`.
   - Records the previous active pointer, switches via `set_active_layout`, persists, and reports
     `previous_active` in both the JSON envelope and the audit row.

3. **`cmd_fs_layout_remove(rest, store_path_opt, is_json, ctx)`** — `aiosh layout remove <id> [--store <path>] [--json]`
   - Delegates to `FilesystemLayoutStore::remove_layout`, which refuses to delete the currently
     active layout or the built-in canonical presets (`aios-uefi-standard-v1`,
     `aios-container-minimal-v1`) with exit code `1`.

4. **`cmd_fs_layout_import_fstab(rest, store_path_opt, is_json, ctx)`** — `aiosh layout import-fstab <id> <name> --fstab <file_or_content> [--base <id>] [--store <path>] [--json]`
   - Accepts fstab content either inline or as a file path (10 MiB ceiling).
   - Parses via `FilesystemLayoutService::import_fstab_as_layout`, inheriting partitions and
     directories from `--base` (default: the active layout), and registers the resulting profile.

All four paths keep the subsystem invariants: every command emits exactly one structured audit row
through `classify_and_emit` on success and on failure, and every mutation is persisted through the
crash-safe `.tmp.<pid>` + atomic-rename path.

### 1.1 Defect found and fixed during implementation

The `--spec` guard introduced with the scaffold rejected **inline JSON specifications**. The guard
applied path rules (`len() <= 1024` and no control characters) to the raw argument, but
pretty-printed layout JSON legitimately spans multiple lines (each newline is a control character)
and routinely exceeds 1024 bytes. This made the documented `--spec <file_or_json>` contract
unusable for the inline form and produced a spurious exit code `2`.

Fixed by scoping the guard to what it actually protects:

- `--spec` values are now rejected up front only when they contain a NUL byte; path-style values
  are still length-bounded and size-capped before any read.
- The same correction was applied to the `resolve_layout` closure so `show`, `validate`, `probe`,
  and `fstab` accept inline JSON specs consistently with `register`.

Note that `validate --spec <invalid-json>` previously returned exit code `1` for the wrong reason
(the guard error, not a validation failure). It now returns `1` because the spec genuinely fails
`FL1` validation.

---

## 2. Test Execution & Verification

Targeted CLI flow test (extended to cover the full persistent-store lifecycle — register,
duplicate rejection, list, show, set-active, active-removal refusal, fstab import, cross-profile
diff, re-point, removal, repeat-removal, built-in protection):

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml --bin aiosh test_cmd_fs_layout_flow
   Compiling aiosh-cli v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 29.78s
     Running unittests src\main.rs

running 1 test
test task_cli_tests::test_cmd_fs_layout_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.81s
```

Regression: full `aiosh-cli` binary suite.

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 105.20s
```

Regression: untouched `fs_layout` core suites (data model + core service).

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_fs_layout_data_model --test test_fs_layout_service
running 19 tests
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

Regression: MCP crate (peer surface of the same subsystem).

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.79s
```

Regression: end-to-end MCP stdio smoke suite.

```
$ python code/aiosh-mcp/tests/test_session_mcp_smoke.py
PASS: tools/list contains all 6 aios.session.* tools
PASS: aios.session.validate (valid, invalid, boundary, missing)
PASS: aios.session.list & aios.session.get (filtering, inspection, error modes)
PASS: aios.session.action (PEP enforcement, lock, unlock, unknown action, missing params)
PASS: aios.session.create & persistence (PEP enforcement, create, get, duplicate rejection, invalid spec)
PASS: Cross-surface CLI <-> MCP parity & state sharing
PASS: aios.session.check validation and quarantine recovery
PASS: MCP session hardening (payload limits, query bounds, ID injection, store path sanitization)

ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!
```

---

## 3. Acceptance Verification

- [x] Targeted test passes (`test_cmd_fs_layout_flow`, 1/1).
- [x] No regression in existing smoke suites for touched modules (CLI 24/24, MCP 12/12,
      fs_layout data model 19/19, fs_layout core service 11/11, MCP stdio smoke all PASS).
- [x] Working tree compiles cleanly under `cargo check --bin aiosh` and `cargo test`.

---

## 4. Known Limitations

- Inline `--spec` content is bounded only by the 10 MiB read ceiling for files; the inline form is
  not additionally length-capped beyond operating-system `argv` limits.
- `register` does not yet reject `..` traversal segments inside a `--spec` path argument; the value
  is only ever read as UTF-8 JSON, and the dedicated security review is scheduled as `T-01527`.
