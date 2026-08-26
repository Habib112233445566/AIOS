# T-00121 — CI Smoke Orchestration / core service: Research

Date: 2026-08-23 · Status: RESEARCH COMPLETE — no code changed.

## Scope

Core-service lifecycle for CI Smoke Orchestration: the service layer that
CONSUMES what the T-00111..T-00120 data model produces (the run summary
artifact), turning it into operator/agent-answerable questions.

## F1 — Existing implementation inventory (FACT)

- Producer side complete: `tools/ci_suites.py` (registry, record
  constructors, atomic `write_summary`) and `tools/ci_run.py`
  (orchestrator writing `$AIOSH_CI_RESULTS`, default
  `/tmp/aiosh-ci-results.json`).
- Consumer side: NOTHING. The only consumers today are ad-hoc
  `python3 -c` snippets pasted into docs (README §CI example).

## F2 — Identified gaps (FACT)

1. **G1 No consumer service.** Answering "what failed last run?" requires
   hand-written JSON munging every time.
2. **G2 No gate semantics.** Nothing exposes "did the last full CI run
   pass?" as an exit code other pipelines can consume.
3. **G3 No staleness/validity handling.** A truncated, hand-edited, or
   schema-version-mismatched artifact is silently trusted by any future
   consumer.
4. **G4 No failure-focused projection.** Failed suites' log paths exist in
   records but there is no one-command view linking suite → log → tail.
5. **G5 No partial-run awareness.** Fail-fast runs record fewer results
   than SUITES; consumers must infer "incomplete" themselves
   (`total != len(SUITES)`).

## Assumptions vs facts

- FACT: all gaps verifiable by absence-of-code plus the README example.
- ASSUMPTION: a read-only CLI service (no daemon, no MCP surface yet)
  matches current need; MCP exposure of orchestration was explicitly out
  of scope in T-00112 §6. Confirm at T-00122 review.
- ASSUMPTION: `schema_version` mismatch should refuse loudly rather than
  best-effort parse (repo-wide loud-error rule).

## Decisions needed before implementation

- D1: new `tools/ci_service.py` exposing read-only actions over the
  artifact: `show` (human report), `failures` (failed/timeout/error rows
  + log tails), `check` (gate: exit 0 iff last recorded run was
  all_pass AND complete).
- D2: validate on load: schema_version == 1, required key set present,
  per-record shape sane; ANY violation ⇒ loud refusal naming the field.
- D3: exit codes: 0 ok/all-pass · 1 gate-failed · 2 loud error (usage,
  missing/corrupt artifact).
- D4: `AIOSH_CI_RESULTS` honored identically to the producer (single env
  contract); `--file PATH` flag as explicit override.
- D5: stdlib only; no audit-ring interaction (host tooling, consistent
  with T-00112 §6).
