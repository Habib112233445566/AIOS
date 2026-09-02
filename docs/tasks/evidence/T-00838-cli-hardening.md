# T-00838 — Regression Triage / CLI: Hardening

## 1. Hardening Deliverables
- **Input Validation**: Verified strict requirement of mandatory parameters for `record` (`--target`, `--error`), `show` (positional `<id>`), `resolve` (positional `<id>` and `--notes`), and `ingest` (positional `<summary_file>`).
- **Fail-Closed Result Codes**: Unhandled flags or unrecognized subcommands consistently return exit code 2.
- **Auditable Failures**: Storage errors and validation failures write explicit diagnostics to stderr.
