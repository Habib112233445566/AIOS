# T-00836 — Regression Triage / CLI: Integration

## 1. Integration Deliverables
- Integrated `cmd_triage` into main executable binary `aiosh` with full CLI help discovery.
- Connected modifying subcommands (`record`, `resolve`, `ingest`) to SQLite WAL audit trail via `classify_and_emit`.
- Verified end-to-end integration via `tools/test_triage_suites.py` criterion T3.
