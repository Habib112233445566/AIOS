# T-00191 — CI Smoke Orchestration / documentation: Research

**Date:** 2026-08-26
**Feature:** CI Smoke Orchestration documentation

## 1. Scope & Objective
Establish facts, constraints, and prior art for the documentation of the CI Smoke Orchestration domain. Identify gaps where the CI orchestrator, configuration limits, and MCP/CLI actions are under-documented.

## 2. Facts vs. Assumptions

### F1: Current Documentation (FACTS)
- The root `README.md` briefly mentions `ci/run_all_smokes.sh` as the script to run before and after code changes.
- The `aiosh ci` CLI commands (`show`, `failures`, `check`, `config`, `metrics`) are entirely missing from the "Supported Surfaces" or "Usage" sections of the `README.md` or any central spec document.
- The Twelve-Factor configuration knobs (`AIOSH_CI_MAX_FILE_BYTES`, `AIOSH_CI_TIMEOUT_DEFAULT_S`, etc.) are documented in their specification evidence files (`T-00152-spec.md`), but not easily discoverable by a new user reading the repository root.

### F2: The Gap (FACTS)
- New developers or AI agents booting into this repository do not know that `aiosh ci metrics` exists or that they can tune the CI timeout by setting `AIOSH_CI_TIMEOUT_DEFAULT_S`.

## 3. Prior Art & Constraints
- Documentation updates typically focus on `README.md` and dedicated markdown specifications (`docs/SPEC-*.md`).
- We must respect the repo's existing `README.md` format, inserting CLI surface documentation near the "Supported Surfaces" or "Task ledger" sections.

## 4. Decisions Needed Before Implementation
- **D1 (Location):** Should we append a dedicated "CI Smoke Orchestration" section to `README.md` or create a `docs/SPEC-CI.md`?
  - *Decision:* Given that CI is foundational to the "No-Skip" task laws, adding a concise block to `README.md` detailing the CLI commands and environment variables is best for discoverability.
- **D2 (Content):** We must document the `aiosh ci` commands, the orchestrator script `ci/run_all_smokes.sh`, and the configuration knobs (`AIOSH_CI_*`).
