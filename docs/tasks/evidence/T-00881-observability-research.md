# T-00881 — Regression Triage / Observability: Research

## 1. Prior Art & Observability Landscape
- **Data Model Metrics (`aiosh-core::triage`)**:
  - `TriageReport` tracks `total_records`, `open_records`, `resolved_records`, and `generated_at`.
  - Granular breakdown helpers needed: `status_counts()`, `severity_counts()`, and standardized `summary_line()`.
- **Audit Ring Integration**:
  - Consequential triage mutations (`record`, `resolve`, `ingest`) emit structured audit entries into SQLite WAL.
- **CLI & MCP Telemetry**:
  - `aiosh triage check` and `aios.triage.check` return quantitative summary status and telemetry diagnostics.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Report Metrics | Fact | `TriageReport` aggregates failure counts and lifecycle distributions. |
| Diagnostic Line | Fact | `summary_line()` provides structured string for stdout/stderr and logs. |
| Audit Trail | Fact | All mutations route through `classify_and_emit` with actor and outcome metadata. |

## 3. Decisions & Actions
- Implement `status_counts()`, `severity_counts()`, and `summary_line()` in `aiosh-core::triage`.
- Add criterion `T7` in `tools/test_triage_suites.py`.
