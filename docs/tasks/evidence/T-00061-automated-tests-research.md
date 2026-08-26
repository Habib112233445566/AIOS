# T-00061 — Task Ledger Control: automated tests Research

**Date:** 2026-08-22
**Type:** research (no code changed)
**Depends on:** T-00060
**Artifact note:** instruction name `T-00061-research.md`; ledger row
declares `T-00061-mcp-api-surface-research.md`? No — row declares
`T-00061-automated-tests-research.md` (mirrored).

Central question: with five wire/unit suites already pinning the ledger
(U/W/C/P/K), what test gaps remain, and which are worth closing in
this sub-epic?

## 1. Internal facts (inventory + probes, 2026-08-22)

| # | Fact |
|---|---|
| F1 | Current suites & counts: Rust workspace 79 unit tests (ledger 10+, task_service 13+, ledger_config, audit/pep/retention/pentest/sandbox/agent/canonical); Python U1..U16 (legacy module+CLI), W1..W8 (Rust MCP wire), C1..C9 (CLI wire), P1..P8 (Python MCP wire), K1..K5 (config wire). CI runs all: 14 suites. |
| F2 | **Glob semantics (probed by read):** `tool_glob_match` supports exact match and trailing `.*` prefix only — so grant `--tools "aios.*"` DOES authorize `aios.task`, but `"aios.task*"` would also match `aios_taskX`-style names; no mid-string wildcards. Untested for the task surface. |
| F3 | **Untested cross-surface interactions identified:** (a) a wildcard grant `aios.*` on the PYTHON server; (b) evidence-cap violation over the Python MCP surface (only note oversize tested there); (c) lock-contention behavior through the MCP tool path (unit-tested only); (d) `task config` refusal audited-row shape (K4 covers CLI only). |
| F4 | Test-style conventions are established and stable: repo PASS/FAIL markers, isolated sandboxes, observable-behavior assertions, broken-feature checks (T-25/T-35 precedent). |
| F5 | CI wiring pattern proven three times (task_service/task_cli/task_config smokes) — adding a suite is mechanical. |

## 2. External authoritative facts

Reused: MCP E1–E5 (T-00021), POSIX G-guidelines (T-00031),
Twelve-Factor (T-00051) — none introduce new obligations for TESTS.
No new external sources required; prior citations remain valid
(fetched 2026-08-21/22).

## 3. Gap analysis → candidate scope

The component should consolidate scattered coverage into ONE
cross-surface regression suite that pins the interactions no single
surface test can see:

Candidate A (recommended proposal, AIOS-specific): new
`test_ledger_matrix_smoke.py` covering:
1. wildcard-grant authorization (`aios.*`) on BOTH servers;
2. overly-narrow grant (`aios.task.done` exact-string) REJECTED;
3. evidence-item cap over the Python MCP surface;
4. lock-contention via two concurrent CLI writers → one succeeds,
   one loud lock-busy (integration-level, bounded 5s);
5. config knob actually changes MCP-side validation (env override +
   oversize note over MCP).
Rejected alternatives: property/fuzz testing (no framework in deps;
out of proportion), coverage-percentage tooling (tmcfg not installed;
behavioral suites are the project convention).

## 4. Assumptions (marked)
A1: five-second lock timeout makes test #4 deterministic enough (~≤6 s).
A2: FastMCP direct-fn invocation remains an acceptable proxy for wire
calls for the Python leg (established by P-suite precedent).

## 5. Decisions needed before Specification (T-00062)

- **D1:** scope = Candidate A's five cases? (default yes)
- **D2:** single new suite vs extending existing four? (default: one
  new matrix suite, wired into CI at Integration step)
- **D3:** keep per-surface suites untouched (no consolidation/refactor)?
  (default yes — they encode per-surface contracts)

## 6. Acceptance check
- [x] Facts (F1–F5) separated from assumptions (A1–A2); decisions D1–D3.
- [x] Citations carried forward; no fabrication needed.
- [x] No code changed.
