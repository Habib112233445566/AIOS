# T-00161 — CI Smoke Orchestration / automated tests: Research

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration automated tests

## 1. Scope & Objective
Establish facts, constraints, and prior art for testing the CI Smoke Orchestration components. The focus is to identify untested or under-tested components in the CI pipeline and define a testing strategy to close those gaps.

## 2. Facts vs. Assumptions

### F1: Current Test Coverage (FACTS)
- **Data Model:** `tools/ci_suites.py` is tested by `tools/test_ci_suites.py` (W1-W7 coverage: atomic writes, registry validity, error propagation).
- **Core Service:** The JSON artifact parser and strict schema validation (`load_summary_with_retry`) are tested cross-substrate via `tools/test_ci_service.py` (X1-X7).
- **Configuration:** Twelve-Factor environment knobs (`CiConfig`) are verified by `tools/test_ci_config.py`.
- **MCP API:** The schema boundaries are covered in `test_ci_mcp_smoke.py`.

### F2: The Gap (FACTS)
- **Untested Core Runner:** The orchestrator itself, `tools/ci_run.py`, has **no isolated test suite**. Its logic for fail-fast termination, log truncation (`_print_failure_tail`), process group tree killing on timeout (`_terminate_group`), and sequential contract enforcement is completely unverified.
- **ASSUMPTION:** The `automated tests` epic (161-170) is primarily intended to build an isolated, fast-running unit/smoke test for `tools/ci_run.py` to ensure it honors its timeouts, correctly records statuses, and bounds log tails per the configuration constraints.

## 3. Prior Art & Constraints
- The project's testing convention emphasizes cross-substrate parity where possible, but `ci_run.py` is a pure Python executable driving `subprocess.Popen`.
- Tests must be fast, offline, and require no live networks.
- A test for `ci_run.py` must use mock scripts or `python -c` snippets as "suites" rather than running the real suites to avoid taking minutes.

## 4. Decisions Needed Before Implementation
- **D1 (Test Boundary):** How do we test `ci_run.py` without mutating the canonical `tools/ci_suites.SUITES` array? 
  - *Decision:* The test must mock or dynamically overwrite `ci_suites.SUITES` with fake/fast scripts during execution, or use `unittest.mock.patch` if imported as a module.
- **D2 (Scenarios):** We must cover: 
  - Happy path (all pass).
  - Fail-fast path (first fail stops execution).
  - Timeout path (process group terminated; timeout status).
  - Log tail bounding (ensuring >1MB logs are truncated gracefully).
