# T-00631 — Repository Health / CLI surface: Research

## 1. Problem Statement & Background
The `aiosh` CLI requires a command interface to invoke repository health checks, inspect individual check diagnostics, and output structured report summaries for operators and automated pipelines.

## 2. CLI Surface Architecture & Subcommands

### A. Subcommand Grammar
```bash
aiosh repo <health|check> [--repo <path>] [--json]
```
- `aiosh repo health`: Executes full multi-check suite and prints diagnostic overview.
- `aiosh repo check`: Alias for `aiosh repo health` matching developer convention.
- Flags:
  - `--repo <path>`: Repository root path (default: current working directory `.`).
  - `--json`: Formats output as raw JSON object adhering to `RepoHealthReport` schema.

### B. Exit Code Semantics
- `0`: All checks passed or warned without critical failures (`overall_status == Pass | Warn`).
- `1`: One or more checks failed (`overall_status == Fail`) or repository path unreadable.
- `2`: Invalid CLI subcommand or unknown option flags.

### C. Audit Trail Emission
- Consequential actions write exactly one audit row to the SQLite audit database via `emit(ctx, "repo.health", ...)`.

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Working Assumption |
| :--- | :--- | :--- |
| **CLI Dispatch** | Top-level dispatcher in `aiosh-cli/src/main.rs` routes subcommands to `cmd_<name>` handlers. | Adding `Some("repo") => cmd_repo(&args[1..])` provides backward-compatible expansion. |
| **JSON Output** | `ok_out` and `err_out` print pretty-printed JSON payloads. | Machine consumers can filter or pipe `--json` output directly. |
| **Audit DB** | `open_context()` handles schema preparation and audit row writes. | Read-only CLI checks log audit events under actor `user`. |

## 4. Key Design Decisions for CLI Surface
1. Implement `cmd_repo(args: &[String]) -> i32` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
2. Register `repo` in `main()` dispatcher and help text.
3. Support human-readable summary table and `--json` envelope.
4. Emit structured audit row with tool `repo.health`.
