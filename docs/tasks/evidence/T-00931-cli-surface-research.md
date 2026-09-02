# T-00931 — Agent Handoff Protocol / CLI Surface: Research

## 1. Prior Art & Architecture
- **CLI Commands (`aiosh handoff`)**:
  - `aiosh handoff list [--active] [--status <status>] [--json] [--store <path>]`: Query active and historical handoffs.
  - `aiosh handoff show <id> [--json] [--store <path>]`: Retrieve detailed status and context of a specific handoff.
  - `aiosh handoff initiate --sender <S> --receiver <R> [--task <T>] --summary <CTX> [--payload <JSON>] [--priority <P>]`: Enqueue a new handoff request.
  - `aiosh handoff accept <id> [--notes <N>]`: Transition pending request to accepted.
  - `aiosh handoff reject <id> [--notes <N>]`: Transition pending request to rejected.
  - `aiosh handoff complete <id> [--notes <N>]`: Transition accepted request to completed.
  - `aiosh handoff cancel <id> [--notes <N>]`: Revoke pending/accepted handoff.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Command Dispatch | Fact | Added to `code/aiosh-rust/aiosh-cli/src/main.rs` under subcommand `handoff`. |
| Audit Compliance | Fact | All state-changing subcommands emit exactly one audit row to SQLite WAL. |
| Exit Codes | Fact | Exit code 0 for success, 1 for domain/operational failure, 2 for invalid CLI usage. |

## 3. Decisions & Actions
- Implement `cmd_handoff` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Add criterion `H3` to `tools/test_handoff_suites.py`.
