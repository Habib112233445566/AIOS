# T-00892 — Regression Triage / Documentation: Specification

## 1. Documentation Structure & Specification

The documentation in `docs/README.md` must specify:
1. **Data Model**: `TriageRecord`, `TriageReport`, `TriageStatus`, `TriageSeverity`, and deterministic SHA-256 failure fingerprints.
2. **Service Layer**: `TriageStore`, persistence in JSON format, atomic save operations, and CI test summary ingestion.
3. **CLI Interface**: `aiosh triage list/show/record/resolve/ingest/check` with parameter syntax, flags, and exit codes.
4. **MCP API Surface**: 5 JSON-RPC tools with parameter contracts and SQLite WAL audit logging.
5. **Configuration**: `TriageConfig`, schema validation, bounds (16 KiB .. 64 MiB), and suite filters.
6. **Automated Testing**: `tools/test_triage_suites.py` asserting criteria `T1..T7`, and `tools/test_triage_unit.py` (U01..U08).
7. **Security Policy**: Prohibitions against triage record tampering and vulnerability definitions in `SECURITY.md`.
8. **Observability**: Metrics and single-line summary diagnostics.
9. **Evidence Chain**: Complete link index for tasks `T-00811` through `T-00899`.
