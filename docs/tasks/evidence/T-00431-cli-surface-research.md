# T-00431 — Documentation Index Control / CLI surface: Research

## 1. Goal
Establish facts, command syntax conventions, output formatting requirements, and prior art for the CLI surface of Documentation Index Control (`aiosh doc`).

## 2. Facts vs. Assumptions

### Facts (Empirical Repository Context):
1. **Existing CLI Subcommands**:
   - `code/aiosh-rust/aiosh-cli/src/main.rs` dispatches top-level commands via `clap` (or manual argv routing): `task`, `ci`, `release`, `backup`, `toolchain`, `audit`, `pentest`, `run`.
2. **Output Invariants**:
   - Human-readable output uses clean ASCII status lines (`[+]`, `[-]`, `PASS:`, `FAIL:`) to avoid CP1252 Windows encoding exceptions.
   - Machine-readable `--json` flags output formatted canonical JSON envelopes.
3. **Exit Code Conventions**:
   - Exit 0 on successful validation / display.
   - Exit 1 on validation failure / broken links.
   - Exit 2 on usage / syntax / argument errors.

### Assumptions:
1. `aiosh doc` should provide operators with fast local tools to inspect documentation trees, validate link integrity, and search document sections.
2. CLI execution should interface directly with `aiosh_core::doc_index` and `aiosh_core::doc_index_service`.

## 3. Prior Art & CLI Design Patterns
- **Cargo / Rustdoc CLI**: `cargo doc --open` and documentation search indices.
- **Git Documentation Tools**: `git help` / `git doc`.
- **Markdown Link Checkers**: `markdown-link-check` emitting structured failure summaries.

## 4. Proposed CLI Command Matrix
1. `aiosh doc show [--json]`: Displays catalog of indexed documentation entries grouped by section.
2. `aiosh doc check [--repo <path>] [--json]`: Runs link resolution against markdown files and reports broken or out-of-bounds references.
3. `aiosh doc search <query> [--json]`: Queries indexed titles, sections, and paths for matching keywords.

## 5. Decisions Needed
1. **Command Prefix**: Use `aiosh doc <subcommand>`.
   - *Decision*: Adopt `aiosh doc` as the top-level command name.
2. **Default Discovery**: In the absence of an explicit path, discover docs relative to the active repository root.
   - *Decision*: Default `--repo` to `.` (current working directory / repo root).

## 6. Next Steps
Advance to Specification (T-00432) to formalize the CLI arguments, flags, exit codes, and stdout/stderr contracts.
