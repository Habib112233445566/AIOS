# T-00845 — Regression Triage / MCP/API: Unit Test

## 1. Unit Test Deliverables
- Validated `tools/test_triage_suites.py` criteria T1..T4 in isolation.
- Validated `test_mcp_triage_tools` in `aiosh-mcp`:
  - `aios.triage.list` on empty store returns count 0 and `ok: true`.
  - `aios.triage.record` stores record with `id` and `signature`.
  - `aios.triage.show` returns record metadata.
  - `aios.triage.check` detects open critical regression and returns `clean: false`.
  - `aios.triage.resolve` updates status to resolved.
  - Subsequent `aios.triage.check` returns `clean: true`.

## 2. Test Execution Output
```text
[+] T1 triage data model integrity & failure signatures
[+] T2 triage store, CI summary ingestion & persistence
[+] T3 CLI surface commands, flags & flow
[+] T4 MCP surface tools, params & flow

PASS: triage_suites criteria (T1..T4)
```
