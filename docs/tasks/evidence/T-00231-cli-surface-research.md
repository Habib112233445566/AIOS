# T-00231 — Phase 0 — Release Packaging & Backup / CLI surface: Research

## Goal
Establish facts, constraints, and prior art for the CLI surface of Release Packaging & Backup.

## Facts
- **Entrypoint**: The CLI surface is located in `aiosh-rust/aiosh-cli/src/main.rs`.
- **Parsing Mechanism**: Arguments are parsed manually (e.g., using `args.iter()` and a helper `parse_flag()` function) without an external framework like `clap`.
- **Context Injection**: All CLI functions initialize `Ctx` via `open_context()`, which contains the `AuditRing`, `PepStore`, constitution references, and the user's `actor_id`.
- **Core Abstractions**: `aiosh_core::release::generate_release` and `aiosh_core::release::create_backup` take a `ReleaseCtx` (which encapsulates `&mut AuditRing`, `actor_id`, and `constitution_rev`).
- **Standard Audit Envelope**: `emit(...)` and `classify_and_emit(...)` are historically used in `main.rs` to write to the `AuditRing`. However, the new `release.rs` module manages its own `AuditRowInput` submissions, meaning the CLI wrapper does *not* need to explicitly invoke `emit()` for these operations—the core service does it safely.

## Assumptions
- **Subcommand Hierarchy**: It is assumed we will introduce two top-level commands: `aiosh release generate` and `aiosh backup create` (mirroring MCP `aios.release.generate` and `aios.backup.create`).
- **Argument Types**: We assume boolean flags (e.g., `--include-audit`, `--include-memory`) can be parsed as presence flags or string comparisons (`true`/`false`).
- **List Arguments**: We assume `--components` can be a comma-separated string that the CLI tokenizes into a `Vec<String>`, rather than requiring repetitive `--component a --component b` flags.

## Unknowns & Decisions Needed
1. **Command Grouping**: Should `release` and `backup` be separate top-level words (e.g., `aiosh backup` vs `aiosh release`), or should they be unified under a system command?
   *Decision Required*: Proceed with `aiosh release generate` and `aiosh backup create` for 1:1 parity with the MCP namespace.
2. **Boolean Inputs**: Should `--no-audit` be used instead of `--include-audit false` for better CLI ergonomics?
   *Decision Required*: Prefer `--include-audit` / `--no-audit` style boolean toggles to maintain consistency with standard CLI patterns.
3. **Array Inputs**: How should `--components` be formatted in CLI? 
   *Decision Required*: Parse as a comma-separated list (e.g. `--components aiosh-mcp,aiosh-rust`).

## Acceptance Criteria Verified
- [x] Evidence file exists and separates facts from assumptions.
- [x] No code changed; decisions needed are listed explicitly.
