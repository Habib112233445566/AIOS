# T-01522: Filesystem Layout - CLI Surface: Specification

## Metadata
- **Task ID:** `T-01522`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface (`code/aiosh-rust/aiosh-cli::cmd_fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (2/10) — CLI Surface Specification
- **Dependencies:** `T-01521` (CLI Surface Research)
- **Next Task:** `T-01523` (Filesystem Layout / CLI surface: Scaffold)

---

## 1. Scope & Objective

This specification formalizes the command syntax, options, input resolution, exit codes, audit effects, and JSON output envelopes for the operator command-line interface:
- Primary Command: `aiosh layout`
- Compatibility Alias: `aiosh fs-layout`

---

## 2. Reused vs. New Interfaces

### 2.1 Reused Subsystem Interfaces
- `FilesystemLayoutService` & `FilesystemLayoutStore` ([`fs_layout_service.rs`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/fs_layout_service.rs)):
  - Storage registry, probing, layout diffing, fstab synthesis, and atomic file persistence.
- `FilesystemLayoutSpec`, `MountPointSpec`, `PartitionSpec`, `DirectorySpec` ([`fs_layout.rs`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/fs_layout.rs)):
  - Invariants FL1..FL5, presets `standard_uefi` and `minimal_container`.
- `classify_and_emit`, `open_context`, `has_flag`, `parse_flag` ([`aiosh-cli/src/main.rs`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-cli/src/main.rs)):
  - CLI context, argument extraction, rule-pack classification, and SQLite WAL audit logging.

### 2.2 New CLI Subcommands Specified
1. `list`: Lists registered layout profiles with active pointer indication.
2. `show`: Displays full layout specification (partitions, mounts, directories).
3. `validate` / `check`: Verifies consistency invariants FL1..FL5.
4. `probe`: Evaluates block device size feasibility against partition budgets.
5. `diff`: Computes differential comparison between source and target layouts.
6. `fstab`: Emits standard 6-field `/etc/fstab` text.
7. `register`: Registers a new layout spec into persistent store.
8. `set-active`: Switches the active layout pointer.
9. `remove`: Removes a non-active custom layout profile.
10. `import-fstab`: Imports external fstab content into a new layout profile.

---

## 3. Subcommand Specifications

### 3.1 `aiosh layout list`
- **Arguments**: `[--store <path>] [--json]`
- **Behavior**: Reads layout store from `--store` (or default `/etc/aios/fs_layouts.json` or fallback memory store). Emits table or JSON array of profiles.
- **Exit Codes**: `0` on success, `2` on invalid store path.
- **Audit**: `fs_layout.list` (count, active ID).

### 3.2 `aiosh layout show [ID]`
- **Arguments**: `[ID] [--standard|--container|--spec <F>] [--store <path>] [--json]`
- **Behavior**: Resolves layout (from ID, spec file, container preset, or active layout default) and displays configuration.
- **Exit Codes**: `0` on success, `1` if layout ID not found, `2` on syntax error.
- **Audit**: `fs_layout.show` (layout ID, partition count, mount count).

### 3.3 `aiosh layout validate`
- **Arguments**: `[--standard|--container|--spec <F>] [--store <path>] [--json]`
- **Behavior**: Runs `validate_filesystem_layout` against target spec.
- **Exit Codes**: `0` if valid, `1` if invalid (with violation message), `2` on syntax error.
- **Audit**: `fs_layout.validate` (valid: bool, error).

### 3.4 `aiosh layout probe`
- **Arguments**: `[--bytes <N>] [--standard|--container|--spec <F>] [--store <path>] [--json]`
- **Behavior**: Evaluates if `<bytes>` satisfies minimum target disk bytes and partition allocation sum.
- **Exit Codes**: `0` if viable, `1` if insufficient, `2` on invalid byte count.
- **Audit**: `fs_layout.probe` (is_viable: bool, errors count, warnings count).

### 3.5 `aiosh layout diff [SRC] [TGT]`
- **Arguments**: `[SRC] [TGT] [--store <path>] [--json]`
- **Behavior**: Computes delta across partitions, mounts, and directories. Highlights `destructive: bool`.
- **Exit Codes**: `0` on success, `1` if either layout ID not found.
- **Audit**: `fs_layout.diff` (source, target, destructive).

### 3.6 `aiosh layout fstab`
- **Arguments**: `[--standard|--container|--spec <F>] [--store <path>] [--json]`
- **Behavior**: Synthesizes clean `/etc/fstab` text with comments and aligned columns.
- **Exit Codes**: `0` on success, `1` on failure.
- **Audit**: `fs_layout.fstab` (layout ID, lines count).

### 3.7 `aiosh layout register --spec <F>`
- **Arguments**: `--spec <file_or_str> [--store <path>] [--json]`
- **Behavior**: Deserializes layout spec, validates FL1..FL5, registers into store, and atomically saves store.
- **Exit Codes**: `0` on success, `1` on validation or duplicate failure, `2` if `--spec` is missing.
- **Audit**: `fs_layout.register` (layout ID, success).

### 3.8 `aiosh layout set-active <ID>`
- **Arguments**: `<ID> [--store <path>] [--json]`
- **Behavior**: Updates `active_layout_id` in store and persists.
- **Exit Codes**: `0` on success, `1` if ID not found, `2` if ID missing.
- **Audit**: `fs_layout.set_active` (new_active_id).

### 3.9 `aiosh layout remove <ID>`
- **Arguments**: `<ID> [--store <path>] [--json]`
- **Behavior**: Removes layout from store and persists. Rejects removing active layout or canonical presets.
- **Exit Codes**: `0` on success, `1` if prohibited or not found, `2` if ID missing.
- **Audit**: `fs_layout.remove` (removed_id).

### 3.10 `aiosh layout import-fstab <id> <name> --fstab <F>`
- **Arguments**: `<id> <name> --fstab <file_or_content> [--base <id>] [--store <path>] [--json]`
- **Behavior**: Parses external fstab, creates new layout, registers, and persists.
- **Exit Codes**: `0` on success, `1` on parse error or mount count > 128, `2` on missing args.
- **Audit**: `fs_layout.import_fstab` (imported_id, mounts_count).

---

## 4. Standard Result Envelopes (`--json`)

### 4.1 Success Envelope
```json
{
  "code": 0,
  "data": { ... },
  "error": null
}
```

### 4.2 Failure Envelope
```json
{
  "code": 1,
  "data": null,
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "FL1 violation: root mount pass number must be 1, found 0"
  }
}
```

### 4.3 Syntax / Argument Error Envelope
```json
{
  "code": 2,
  "data": null,
  "error": {
    "code": "ARGUMENT_ERROR",
    "message": "register requires '--spec <file_or_json>' argument"
  }
}
```

---

## 5. Security & Invariant Guarantees

1. **Path Traversal & Control Character Prevention**: Store and spec paths must be $\le 1024$ characters and contain zero control characters or null bytes.
2. **File Size Bounds**: Spec and fstab files are capped at 10 MiB before reading.
3. **Atomic Persistence**: Modifications to the layout store file are performed via temporary sibling files and atomic replacement.
4. **Audit Immutability**: All commands log structured rows to the SQLite audit log ring before exiting.
