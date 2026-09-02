# T-00906 — Regression Triage / Recovery & Validation: Integration

## 1. Integration Deliverables
- Integrated structural validator `validate_triage_record` into `validate_triage_report` and store ingestion path.
- Integrated `TriageStore::load_or_recover` for resilient file loading in CLI and MCP server startup paths.
- Connected criterion `T8` into standalone runner `tools/test_triage_suites.py`.
- Verified end-to-end integration across all crates.
