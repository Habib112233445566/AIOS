# T-00181 — CI Smoke Orchestration / observability: Research

**Date:** 2026-08-26
**Feature:** CI Smoke Orchestration observability

## 1. Scope & Objective
Establish facts, constraints, and prior art for the observability of the CI Smoke Orchestration domain. The goal is to define how the system can be monitored for performance, throughput, and health, aligning with the `aiosh task metrics` precedent.

## 2. Facts vs. Assumptions

### F1: Current Output Capabilities (FACTS)
- **CI Runner (`tools/ci_run.py`)**: Produces a structured JSON summary artifact (`/tmp/aiosh-ci-results.json` by default) containing timestamps (`started_at`, `finished_at`), counts (`total`, `passed`, `failed`), and boolean status (`all_pass`). Each suite record also tracks `duration_ms`.
- **CLI/MCP Surfaces (`aiosh ci`)**: Exposes `show` (human readable report) and `failures` (list of failures).

### F2: Precedent (`aiosh task metrics`) (FACTS)
- The Task Ledger implemented observability via a dedicated `metrics` action (T-00081..T-00090). It returns a consolidated JSON snapshot combining data from the ledger state (`completed`, `blocked`), audit ring (`rows`, `verify_ok`), and configuration (`lock_timeout_secs`).

### F3: The Gap (FACTS)
- While the raw data for CI observability is already produced by `ci_run.py`, there is no standardized `metrics` action in the `aios.ci` MCP tool or the `aiosh ci` CLI command. Agents and monitors cannot query a stable, machine-readable snapshot of the *latest* CI run health in a single JSON block without parsing the raw artifact themselves.

## 3. Decisions Needed Before Implementation
- **D1 (Action Definition):** Should we introduce a `metrics` action to `aiosh ci` and `aios.ci` that returns a consolidated JSON object (e.g. `{"ok": true, "ci": {"all_pass": true, "passed": 20, "failed": 0, "total": 20}, "config": {...}}`)?
- **D2 (Data Sources):** The metrics snapshot should read from `AIOSH_CI_RESULTS` (the latest artifact) and `CiConfig::from_env()` (the active CI configuration).
- **D3 (Failure Modes):** How should the metrics action behave if the CI artifact does not exist (i.e. CI has never run on this host)? It should likely return nulls or zeroes gracefully, without raising a hard error, so monitors do not crash on fresh instances.
