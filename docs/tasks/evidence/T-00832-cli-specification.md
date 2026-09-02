# T-00832 — Regression Triage / CLI: Specification

## 1. Command Syntax & Options

```text
aiosh triage <subcommand> [options]

Subcommands:
  list                  List all triage records (supports --status, --severity, --json)
  show <id>             Show details of a specific triage record by TRG-xxxxxxxx ID
  record                Record a test regression manually
                        (--target <str>, --suite <str>, --error <str>, --repro <str>, --severity <str>)
  resolve <id>          Resolve a triage record (--notes <str>)
  ingest <file>         Ingest CI RunSummary JSON and record failed test suites
  check                 Check store health; exit 1 if blocker/critical regressions open

Options:
  --store <path>        Path to triage_store.json (default: .aios/triage_store.json)
  --json                Output machine-readable JSON format
  --status <status>     Filter list by status (untriaged, triaged, fix_pending, resolved, wont_fix)
  --severity <sev>      Filter list by severity (blocker, critical, major, minor)
```

## 2. Exit Code Contract
- `0`: Operation succeeded; or `check` passed with 0 open blocker/critical regressions.
- `1`: `check` failed (open blocker/critical regressions found); or runtime error (file not found, invalid JSON).
- `2`: CLI argument validation or syntax error.
