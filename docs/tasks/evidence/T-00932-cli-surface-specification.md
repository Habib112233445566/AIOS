# T-00932 — Agent Handoff Protocol / CLI Surface: Specification

## 1. CLI Commands & Signatures

```text
aiosh handoff list [--active] [--status <pending|accepted|rejected|completed|cancelled|expired>] [--json] [--store <path>]
aiosh handoff show <id> [--json] [--store <path>]
aiosh handoff initiate --sender <agent_id> --receiver <agent_id> [--task <id>] --summary <text> [--payload <json>] [--priority <low|normal|high|urgent>] [--store <path>]
aiosh handoff accept <id> [--notes <text>] [--store <path>]
aiosh handoff reject <id> [--notes <text>] [--store <path>]
aiosh handoff complete <id> [--notes <text>] [--store <path>]
aiosh handoff cancel <id> [--notes <text>] [--store <path>]
```

## 2. Exit Codes & Invariants
- `0`: Success, formatted table or JSON emitted.
- `1`: Operation rejected (invalid transition, missing record, IO failure).
- `2`: Bad CLI arguments or missing required flags.
- Audit emission: `handoff.initiate`, `handoff.accept`, `handoff.reject`, `handoff.complete`, `handoff.cancel` logged to SQLite WAL.
