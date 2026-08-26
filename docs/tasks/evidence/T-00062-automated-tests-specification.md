# T-00062 — Task Ledger Control automated tests: Specification

**Date:** 2026-08-22
**Type:** specification (no code changed)
**Depends on:** T-00061 research
**Status:** SPECIFIED — D1–D3 locked to research defaults (standing
autonomy, 2026-08-22). AIOS-specific suite design; behavioral-test
conventions per repo precedent (T-25/T-35).

## 1. Resolved decisions

D1 scope = the five matrix cases below. D2 = ONE new suite
(`test_ledger_matrix_smoke.py`, under `code/aiosh-mcp/tests/` since it
drives both MCP substrates + CLI). D3 = existing suites untouched.

## 2. Suite contract: `test_ledger_matrix_smoke.py`

Sandbox per run (temp `AIOSH_TASKS_DIR`/`AIOSH_HOME`; grants minted via Rust CLI):

| Case | Surface(s) | Asserts |
|---|---|---|
| M1 | Python MCP | wildcard grant `"aios.*"` authorizes `done` |
| M2 | Rust MCP (wire) | same wildcard grant authorizes `aios.task status` — SAME grant object, both servers |
| M3 | Both | exact-string grant `"aios.task.done"` is NOT matched by glob logic → mutation refused at pep gate on both surfaces |
| M4 | Python MCP | evidence-list cap (>16) refused pre-gate with envelope error |
| M5 | CLI ×2 concurrent | two simultaneous mutating commands: exactly one ok; the other exits 1 with `ledger lock busy`; ledger ends consistent (`task check` ok, pointer advanced by exactly one) |
| M6 | CLI→MCP env | `AIOSH_LEDGER_MAX_TEXT=64` in effect: 100-char note refused over the PYTHON MCP surface too (config reaches every substrate) |

Exit non-zero on any failure; PASS marker line `PASS: task ledger matrix smoke (M1..M6)`.

## 3. Reused vs new

Reused: sandbox/grant patterns from P/C/K suites; `_dispatch` gate;
`task_ledger` module; Rust binaries. New: one test file; CI entry.
No new dependencies. No production-code changes expected — if any M-case
exposes a defect, fix lands in THIS sub-epic's Implementation task
(T-00064) per the epic lifecycle… **note:** this component IS "automated
tests", so a failing M-case fixes land directly in T-00064/T-00065 with
the no-skip law unchanged.

## 4. Failure matrix
Any unmet assertion → suite exit 1 with `[✗] M<n>` and detail line.
Broken-feature check required at Unit-Test step (sabotage → [✗]).

## 5. Reviewability check
Cases table + conventions above are reviewable without implementation.
