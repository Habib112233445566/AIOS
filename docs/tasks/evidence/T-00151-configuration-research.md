# T-00151 — CI Smoke Orchestration / configuration: Research

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration configuration

## 1. Scope & Objective
Establish facts, constraints, and prior art for the configuration of the CI Smoke Orchestration domain. This includes both the Python orchestrator (`ci_run.py`) and the Rust native service/CLI surface (`aiosh ci`), identifying hardcoded constants and determining how they should be exposed as tunable configuration knobs.

## 2. Facts vs. Assumptions

### F1: Existing Hardcoded Knobs
- **FACT (Python):** `tools/ci_suites.py` hardcodes `DEFAULT_TIMEOUT_S = 900`, `RUST_SMOKE_TIMEOUT_S = 1800`, and `LOG_TEMPLATE = "/tmp/aiosh-ci-{name}.log"`. It respects `AIOSH_CI_RESULTS` for the summary artifact path, defaulting to `/tmp/aiosh-ci-results.json`.
- **FACT (Rust):** `aiosh-core/src/ci.rs` hardcodes a `1MB` max file size cap (`1 * 1024 * 1024` bytes) and bounded retry logic (`3` max retries, `500ms` sleep). It also honors `AIOSH_CI_RESULTS`.

### F2: Prior Art (Task Ledger Control)
- **FACT:** The `aiosh task` domain established a configuration precedent in Epic T-00051..T-00060. It uses namespaced environment variables (`AIOSH_LEDGER_*`) for overrides and exposes an `aiosh task config` subcommand to print the resolved values and their source (`default` vs `env`).

### F3: Assumptions
- **ASSUMPTION:** The CI Orchestration domain should follow the task ledger's K1-K5 design patterns: domain-prefixed env vars (e.g., `AIOSH_CI_*`), bounds validation (floors/ceilings), and loud refusal on invalid values, along with a transparent `aiosh ci config` dump action.
- **ASSUMPTION:** Since the orchestrator is currently Python and the core service CLI is Rust, the configuration resolution logic will need to be strictly mirrored across both substrates.

## 3. Decisions Needed Before Implementation

- **D1 (Knob Taxonomy):** Which constants must become formal configuration variables? Proposed subset:
  - `AIOSH_CI_RESULTS` (existing)
  - `AIOSH_CI_MAX_FILE_BYTES` (to override the 1MB cap safely)
  - `AIOSH_CI_TIMEOUT_DEFAULT_S` (for tweaking the orchestrator's default limit)
  - `AIOSH_CI_LOAD_RETRIES` (to tweak lock contention loops)
- **D2 (Visibility):** Does the `aios.ci` MCP surface also need a `config` action, or is the `aiosh ci config` CLI command sufficient for transparency?
- **D3 (Registry Migration):** Is the suite registry array (`tools/ci_suites.py`) itself considered "configuration" for this epic? Or does this epic focus exclusively on the scalar execution/service tuning knobs mentioned in D1? (Assumption: Only the scalar execution knobs are in scope, matching the Task Execution configuration epic).
