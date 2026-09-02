# T-00821 — Regression Triage / core service: Research

## 1. Prior Art & Subsystem Objectives
- **Context & Goal**:
  - `Regression Triage / core service (T-00821..T-00830)` provides regression ingestion, correlation, and resolution management in userspace.
  - Ingests test run summaries (`aiosh-core::ci::RunSummary`) and failure records.
  - Correlates incoming test failures with existing triage items using `compute_failure_signature`.
  - Automatically updates recurrence counts and observation timestamps for known regressions while instantiating new records for novel failures.
- **Service Architecture**:
  - `TriageStore`: In-memory and disk-persisted repository of triage records.
  - Ingestion Pipeline: Traverses `RunSummary.results`, filters failed runs (`status != "pass"`), parses/extracts failure signatures, and upserts into `TriageStore`.
  - Resolution & State Transition: Supports lifecycle updates (`Triaged`, `FixPending`, `Resolved`, `WontFix`) with timestamped notes.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Ingestion Source | Fact | `ci::RunSummary` provides standard structured results including exit codes, durations, suite names, and log paths. |
| Deduplication Key | Fact | Triage records are deduplicated by their hex SHA-256 `signature`. |
| Persistence Strategy | Fact | `TriageStore` persists as formatted canonical JSON with bounded file size caps (1 MiB default). |

## 3. Decisions & Contracts Needed
1. Specify `TriageStore` and ingestion API in `docs/tasks/evidence/T-00822-core-service-specification.md`.
2. Implement `triage_service.rs` in `aiosh-core` and wire into `lib.rs`.
3. Add criterion `T2` to `tools/test_triage_suites.py`.
