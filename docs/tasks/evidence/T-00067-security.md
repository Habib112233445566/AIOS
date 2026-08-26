# T-00067 — Task Ledger Control automated tests: Security Review

**Date:** 2026-08-22
**Type:** security review of the TEST SUITES themselves (no code changed)
**Depends on:** T-00066

Question for a test-component: do the suites themselves leak state,
credentials, or bypass policy?

## 1. Verified controls

| # | Control | Result |
|---|---|---|
| S1 | **Isolation.** Every ledger suite sandboxes `AIOSH_TASKS_DIR`/`AIOSH_HOME` to temp dirs before any tool invocation; matrix suite applies them to `os.environ` for in-process legs. No suite writes outside its sandbox (M1/M2/P2/C2 assertions observe only sandbox paths). | PASS |
| S2 | **No credential handling.** Grants minted in-suite are throwaway, TTL ≤600 s, scoped to `aios.task`/`aios.*`; no real secrets anywhere in test code. | PASS |
| S3 | **No policy bypass surface.** Suites exercise the public gates only (CLI argv / MCP tools / direct registered fns); no test imports private bypass helpers. | PASS |
| S4 | **Clean-env robustness.** With ALL `AIOSH_*` vars stripped, `aiosh task check` still succeeds via the ancestor-walk resolver — suites (and operators) don't depend on leaked env. | PASS (probed) |
| S5 | **CI-only side effects.** Suites run sequentially under `ci/run_all_smokes.sh`; temp dirs are mktemp-scoped; the M5 holder self-terminates (6 s sleep) and the suite waits on it. | PASS |

## 2. Notes

- The matrix harness mutates `os.environ` for in-process legs —
  process-local by definition; documented in-suite.
- Residual: none open.

## 3. Verdict
**No open bypass.** Acceptance criteria met.
