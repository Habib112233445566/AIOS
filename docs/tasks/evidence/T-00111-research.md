# T-00111 — CI Smoke Orchestration / data model: Research

Date: 2026-08-23 · Status: RESEARCH COMPLETE — no code changed.

## Problem space

Phase 0 now runs **19 smoke suites** on every change. The orchestrator is
a single bash script whose only machine-readable output is its exit code;
results live in human memory and `/tmp` logs. Any consumer (agents,
dashboards, ledger evidence) must re-parse prose.

## F1 — Current implementation inventory (FACT)

`ci/run_all_smokes.sh` (read in full):

- Sequential-by-contract header comment ("Parallel runs corrupt the
  rebuild … seen 2026-08-21"); `set -euo pipefail`; fail-fast on first
  failure (`return 1` inside `run_suite` → `set -e` exits).
- Per suite: log file `/tmp/aiosh-ci-<name>.log`; on failure prints last
  40 lines; PASS increments a counter. No per-suite duration, no status
  record, no summary artifact.
- 19 invocations in fixed order (rust_smoke first, task_docs_scaffold
  last); host repair `chmod +x node_modules/.bin/*` before anything runs.
- Grep confirms NO structured orchestration/reporting exists anywhere in
  `ci/` or `tools/`.

## F2 — Prior art (verified sources)

- **TAP v13** (Test Anything Protocol) — fetched live this session:
  test lines `ok/not ok`, plan `1..N`, directives TODO/SKIP, YAMLish
  diagnostics, `Bail out!`. Relevant idea: plan-as-premature-exit-guard
  and directive semantics; heavyweight text protocol otherwise.
  https://testanything.org/tap-version-13-specification.html
- **JUnit XML** — de-facto CI reporting interchange (suites/testcases with
  time attrs) used by virtually every CI system; relevant precedent for
  per-case `time` and suite nesting. (Not pinned to a versioned spec;
  treated as convention.)
- Repo-internal convention: every checker prints `[✓]/[✗] NAME` lines and
  a final `PASS:`/`FAIL:` marker with non-zero exit on failure
  (tools/check_task_docs.py, check_security_policy.py). The data model
  should COMPLEMENT this human surface, not replace it.

## F3 — Identified gaps (FACT)

1. **G1** No machine-readable suite registry — the suite list exists ONLY
   as bash invocation lines; adding/removing a suite cannot be validated
   programmatically (docs suites count is asserted nowhere).
2. **G2** No result records — no status/exit/duration/log-path per suite.
3. **G3** No run summary artifact agents or dashboards can consume.
4. **G4** Fail-fast hides downstream failures with no persisted trace of
   how far a run got beyond grep-ing prose logs.
5. **G5** Timeouts: bash has none — a hung suite hangs CI forever (no
   wall-clock bound exists today).

## Assumptions vs facts

- FACT: gaps above verifiable by reading `ci/run_all_smokes.sh` (single
  file) and absence of matches for report/orchestration symbols.
- ASSUMPTION: consumers want JSON (repo already speaks canonical JSON
  everywhere else) rather than TAP/JUnit XML. To confirm at T-00112 review.
- ASSUMPTION: keeping bash as the executor (vs rewriting in Python/Rust)
  preserves the sequential-contract comment's institutional knowledge.

## Decisions needed before implementation

- D1: introduce `tools/ci_suites.py` — an ordered SUITES registry
  (`{name, command[], timeout_s}`) mirroring the 19 bash invocations
  exactly; single source for any future runner.
- D2: result-record schema `{suite, index, status(pass|fail|error|timeout),
  exit_code, duration_ms, started_at, finished_at, log_path}`; ISO-8601 Z
  timestamps matching repo convention.
- D3: run-summary schema `{tool, host_schema_version, started_at,
  finished_at, total, passed, failed, all_pass, results[]}` written to a
  JSON artifact (path overridable via `AIOSH_CI_RESULTS`, default under
  /tmp); ADDITIVE-ONLY key rule per metrics precedent.
- D4: keep bash executor; registry consumed by a Python reporter wrapper
  first (core-service epic decides whether Python becomes the executor).
- D5: no new dependencies (stdlib subprocess/time/json only).

## Constraint notes

- Suites share state (dist rebuild) — any model MUST document that order
  is part of the contract (index field preserved).
- Evidence for verify tasks quotes PASS lines; keep human markers intact.
