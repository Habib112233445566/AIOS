# T-00332 — Dependency & Toolchain Pinning / CLI surface: Specification

## 1. CLI Surface Contract

### 1.1 Subcommand: `aiosh toolchain check`

**Synopsis:**
```
aiosh toolchain check [--config <path>]
```

**Behavior:**
1. Resolve the `ToolchainManifest` from:
   - `--config <path>` flag (if provided), OR
   - `$AIOSH_TOOLCHAIN_CONFIG` env var (if set), OR
   - `config/toolchain.json` (default fallback).
2. Call `enforce_toolchain(&manifest)`.
3. On success: print JSON envelope `{"ok": true, "subcommand": "toolchain check", "data": <manifest_with_sources>}` to stdout. Exit 0.
4. On error: print JSON envelope `{"ok": false, "subcommand": "toolchain check", "error": "<message>"}` to stderr. Exit 1.
5. Both paths emit an audit row with tool `"toolchain.check"`.

### 1.2 Subcommand: `aiosh toolchain show`

**Synopsis:**
```
aiosh toolchain show [--config <path>]
```

**Behavior:**
1. Resolve the `ToolchainManifest` (same resolution as `check`).
2. Print JSON envelope `{"ok": true, "subcommand": "toolchain show", "data": <manifest_with_sources>}` to stdout. Exit 0.
3. Does NOT call `enforce_toolchain` — read-only inspection of the config.
4. Emits an audit row with tool `"toolchain.show"`.

### 1.3 Fallback (no subcommand / unknown)
Print usage to stderr and exit 2:
```
usage: aiosh toolchain <check|show> [--config <path>]
```

## 2. `--config` Flag Parsing

The `--config <path>` flag is optional. Parsing rules:
- Must appear after the subcommand (`check` or `show`).
- If `--config` is present but no value follows, exit 2 with usage.
- If both `--config` and `$AIOSH_TOOLCHAIN_CONFIG` are set, `--config` wins (explicit flag takes precedence).
- Unknown flags cause exit 2 with usage.

## 3. Error Cases

| Condition | Output | Exit |
|---|---|---|
| Missing config file | `{"ok": false, "error": "toolchain config not found at ..."}` | 1 |
| Malformed JSON | `{"ok": false, "error": "Malformed toolchain config: ..."}` | 1 |
| Empty required field | `{"ok": false, "error": "invalid toolchain config: ... cannot be empty"}` | 1 |
| Binary not found | `{"ok": false, "error": "toolchain binary not found or failed: ..."}` | 1 |
| Version mismatch | `{"ok": false, "error": "toolchain mismatch: expected ... found ..."}` | 1 |
| Unknown subcommand | Usage to stderr | 2 |
| Missing `--config` value | Usage to stderr | 2 |

## 4. Reused vs New

- **Reused**: `ok_out()`, `err_out()`, `emit()`, `open_context()`, `CFlags::default()` — all existing CLI helpers.
- **Reused**: `ToolchainManifest::from_env()` and `ToolchainManifest::from_source()` — existing core config.
- **New**: `--config` flag parsing (AIOS-specific). `aiosh toolchain show` subcommand (AIOS-specific).

## 5. Audit Effects
- `toolchain.check` — emitted on both success and error paths.
- `toolchain.show` — emitted on success and config-load-error paths.
- Both are read-only actions, no PEP grant required.
