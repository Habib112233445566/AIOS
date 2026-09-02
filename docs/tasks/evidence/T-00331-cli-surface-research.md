# T-00331 — Dependency & Toolchain Pinning / CLI surface: Research

## 1. Facts (from shipped code)

### 1.1 Existing CLI Surface
The `aiosh toolchain check` subcommand was implemented in T-00326 and is wired into
[main.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-cli/src/main.rs) (lines 255–325).

**Dispatch path:**
```
main() → match "toolchain" → cmd_toolchain(args) → match "check"
  → ToolchainManifest::from_env()
  → enforce_toolchain(&manifest)
  → ok_out / err_out + emit()
```

### 1.2 Argument Parsing Pattern
The CLI follows the same ad-hoc `match` dispatch used by all other subcommands (`release`, `backup`, `task`, `ci`, `pentest`). There is **no external arg-parser crate** (no `clap`, no `structopt`). All parsing is positional string matching.

Current pattern for `toolchain`:
- `aiosh toolchain check` — the only supported action.
- Any other subcommand (or no subcommand) prints `usage: aiosh toolchain check` to stderr and exits 2.

### 1.3 Output Envelope
Follows the project-wide standard JSON envelope:
- **Success**: `{"ok": true, "subcommand": "toolchain check", "data": {...}}`
- **Error**: `{"ok": false, "subcommand": "toolchain check", "error": "..."}`

Output uses `ok_out()` / `err_out()` helpers from main.rs.

### 1.4 Audit Integration
Both success and error paths emit an audit row via `emit()` with tool name `"toolchain.check"`.

### 1.5 Config Resolution
`ToolchainManifest::from_env()` in [toolchain_config.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/toolchain_config.rs) resolves config from:
1. `$AIOSH_TOOLCHAIN_CONFIG` environment variable (if set).
2. Falls back to `config/toolchain.json` (relative to CWD).

Size-capped at 64 KB. Validates that `rust_version` and `python_version` are non-empty.

## 2. Assumptions
- **No additional subcommands planned** for `aiosh toolchain` beyond `check` in Phase 0. Future phases may add `aiosh toolchain pin`, `aiosh toolchain update`, etc.
- **No `--config` flag** is exposed. Configuration override is exclusively via the `AIOSH_TOOLCHAIN_CONFIG` environment variable, consistent with the `release` and `backup` config resolution patterns.
- **No `--json`/`--text` output toggle**. All output is JSON (matching every other aiosh subcommand).

## 3. Decisions Needed
1. **Should `aiosh toolchain check --config <path>` be added?** Currently the only way to override the config path is via the environment variable. A `--config` flag would be more ergonomic for operators but adds parsing complexity.
   - **Recommendation**: Defer to Phase 1. The env-var approach is consistent with the rest of the CLI.
2. **Should `aiosh toolchain` (no subcommand) print the manifest without enforcing?** (i.e., a read-only "show" mode.)
   - **Recommendation**: Useful but not blocking. Can be added in T-00334 if desired.

## 4. Prior Art in This Repo
- `cmd_task()` follows the same `match sub` pattern with `status`, `done`, `block`, etc.
- `cmd_ci()` follows the same pattern with `show`, `failures`, `check`, etc.
- No subcommand uses an external argument parser; the pattern is consistent.

## 5. Citations
- Source: [main.rs cmd_toolchain](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-cli/src/main.rs#L255-L325)
- Source: [toolchain_config.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/toolchain_config.rs)
- Source: [toolchain_service.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/toolchain_service.rs)
- ADR-0035 §F-2: Honest audit rows for all outcomes.
