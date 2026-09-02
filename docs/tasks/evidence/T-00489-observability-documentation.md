# T-00489 — Documentation Index Control / observability: Documentation

## 1. Documentation Scope
This task documents the observability architecture, telemetry model (`DocIndexTelemetry`), audit logging behaviors, and diagnostic invocation commands for Documentation Index Control in `docs/README.md`.

## 2. Documentation Updates
- Updated `docs/README.md` with:
  - **Observability & Diagnostics** subsection explaining `DocIndexTelemetry` attributes (`total_docs_indexed`, `total_links_checked`, `broken_links_count`, `is_healthy`).
  - Example CLI command (`aiosh doc check --json`) and MCP surface integration (`aios.doc.check`).
  - Audit logging to SQLite WAL.
  - Updated evidence pointer range (`T-00411`..`T-00488`).

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6).
