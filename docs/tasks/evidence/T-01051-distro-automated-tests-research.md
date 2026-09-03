# T-01051 — Distro Selection & Justification / Automated Tests: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Automated Tests

## 1. Objectives & Scope
Research the automated test suite architecture for the complete Distro Selection & Justification capability.
- Formulate criteria D5 for the configuration subsystem (`distro_config`), augmenting existing criteria D1 (data model), D2 (core service), D3 (CLI), and D4 (MCP).
- Verify Python standard-library-only test harness design (`tools/test_distro_suites.py` and `tools/test_distro_unit.py`).
- Define unified end-to-end criteria ensuring all test runners return exit code 0 under standard execution.
- Validate execution time bounds (tests complete in < 60 seconds).

## 2. Test Matrix Criteria (D1..D5)
- **D1 (Data Model)**: Structural invariants, validation bounds, semver checks.
- **D2 (Core Service)**: Registry lifecycle, profile registration, querying, and atomic disk persistence.
- **D3 (CLI Surface)**: CLI commands (`list`, `show`, `evaluate`, `recommend`, `config`) and exit codes.
- **D4 (MCP Surface)**: JSON-RPC tools (`aios.distro.list`, `show`, `evaluate`, `recommend`) and error envelopes.
- **D5 (Configuration)**: Multi-source resolution, environment overrides, provenance reporting, and hardening checks.

## 3. Next Steps
Proceed to `T-01052` (Specification) to define criteria D5 assertions and runner contracts.
