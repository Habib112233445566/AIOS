# T-00831 — Regression Triage / CLI: Research

## 1. Prior Art & Subsystem Objectives
- **Context & Goal**:
  - `Regression Triage / CLI (T-00831..T-00840)` provides operators and CI pipelines with a CLI command: `aiosh triage`.
  - Exposes subcommands: `list`, `show`, `record`, `resolve`, `ingest`, and `check`.
- **Subcommands Contract**:
  - `list`: List all triage records with optional `--status`, `--severity`, `--json`, and `--store <path>`.
  - `show <id>`: Display detailed metadata and stacktrace for a single triage item.
  - `record`: Manually create or increment a regression report (`--target`, `--suite`, `--error`, `--repro`, `--severity`).
  - `resolve <id> --notes <text>`: Mark an existing regression as resolved with notes.
  - `ingest <summary_file>`: Automatically parse and ingest failed test suites from a CI RunSummary JSON file.
  - `check`: Validates triage store health; exits with code 1 if unaddressed Blocker or Critical regressions exist.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| CLI Entrypoint | Fact | Registered in `aiosh-cli::main` under `Some("triage") => cmd_triage(&args[1..])`. |
| Store Location | Fact | Default store file defaults to `.aios/triage_store.json` or path specified via `--store <path>`. |
| Audit Row Invariant | Fact | Modifying commands (`record`, `resolve`, `ingest`) emit structured audit rows to the SQLite WAL ring. |

## 3. Decisions & Contracts Needed
1. Specify CLI command flags, arguments, exit codes, and JSON outputs in `docs/tasks/evidence/T-00832-cli-specification.md`.
2. Implement `cmd_triage` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
3. Add criterion `T3` to `tools/test_triage_suites.py`.
