# T-00861 — Regression Triage / Automated Tests: Research

## 1. Objectives & Context
- **Context**: `Regression Triage / automated tests` (T-00861..T-00870) establishes the comprehensive test suite and validation criteria for the entire Regression Triage subsystem across data models, core store, CLI, MCP tools, and configuration.
- **Current Test Coverage**:
  - `T1`: Data model integrity and deterministic SHA-256 failure fingerprinting.
  - `T2`: `TriageStore` persistence, deduplication, and CI summary ingestion.
  - `T3`: CLI `aiosh triage` subcommands, parameters, and exit codes.
  - `T4`: MCP `aios.triage.*` JSON-RPC tools, validation, and SQLite WAL audit trail.
  - `T5`: Configuration schema, bounds validation, and auto-ingest suite wildcard filtering.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Test Harness | Fact | `tools/test_triage_suites.py` orchestrates test criteria across all layers with clean terminal output. |
| Test Matrix | Fact | Needs comprehensive criterion `T6` covering the full end-to-end lifecycle (ingest -> detect blocker -> health check fail -> resolve -> health check pass -> recurrence reopening). |
| Timeout & Isolation | Fact | Tests must execute within strict timeouts (120s) with isolated ephemeral temp files. |

## 3. Decisions & Next Steps
- Specify criterion `T6` in `T-00862-spec.md` and scaffold/implement in `tools/test_triage_suites.py`.
