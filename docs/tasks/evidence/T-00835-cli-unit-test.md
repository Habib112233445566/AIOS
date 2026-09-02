# T-00835 — Regression Triage / CLI: Unit Test

## 1. Unit Test Deliverables
- Validated `tools/test_triage_suites.py` criteria T1..T3 in isolation.
- Validated `task_cli_tests::test_cmd_triage_flow`:
  - `list` on empty store returns exit 0.
  - `record` with required `--target` and `--error` inserts item and returns exit 0.
  - `check` identifies unresolved critical regression and returns exit 1.
  - `list --json` outputs formatted json list.
  - Invalid subcommand returns exit 2.

## 2. Test Execution Output
```text
[+] T1 triage data model integrity & failure signatures
[+] T2 triage store, CI summary ingestion & persistence
[+] T3 CLI surface commands, flags & flow

PASS: triage_suites criteria (T1..T3)
```
