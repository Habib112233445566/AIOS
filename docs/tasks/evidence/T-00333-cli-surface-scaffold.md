# T-00333 — Dependency & Toolchain Pinning / CLI surface: Scaffold

## Overview
The CLI scaffold for `aiosh toolchain` already exists in `main.rs` (lines 255–325).
This task verifies the scaffold compiles and identifies the extension points for T-334.

## Existing Scaffold
- `cmd_toolchain(args: &[String]) -> i32` dispatches on `args.first()`.
- Currently handles `"check"` and falls through to usage for anything else.
- Uses `open_context()`, `emit()`, `ok_out()`, `err_out()` — all existing helpers.

## Extension Points for T-334
1. Add `Some("show")` arm to the match.
2. Add `--config` flag extraction as a helper function before the match arms.
3. Add `ToolchainManifest::from_path(path)` to `toolchain_config.rs` for direct path loading (bypassing env resolution).

## Verification
Project compiles with zero errors: `cargo build --all` passes.
