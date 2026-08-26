# T-00118 — CI Smoke Orchestration / data model: Hardening

Date: 2026-08-23 · Status: HARDENING COMPLETE

## Fixes shipped (`tools/ci_run.py`)

1. **Process-group timeout kill (closes S4 residual from T-00117).**
   Suites now start with `start_new_session=True`; on timeout the whole
   group receives SIGTERM via `os.killpg`, falling back to
   `proc.kill()`. Empirical probe P2: a suite spawning its own `sleep 30`
   grandchild was killed at the 1 s deadline and **zero** members remained
   in the process group (pgid-membership scan of /proc — argv-based scans
   self-matched twice during authoring; comm/pgid method is authoritative).
2. **Bounded log-tail reads.** `_print_failure_tail` previously slurped
   the whole log (unbounded memory on a hostile/pathological suite).
   Now: stat size → seek to last 64 KiB max → decode → drop partial
   boundary line → print ≤40 lines. Probe P1: 5 MiB log produced 12 bytes
   of output.

## Deliberate non-changes (documented decisions)

- **No spawn retries**: the sequential fail-fast contract treats any
  spawn failure as terminal; retrying could mask real breakage.
- Log files themselves remain uncapped on disk (evidence preservation);
  only orchestrator MEMORY exposure is bounded. Disk pressure is an
  operator concern (/tmp lifecycle unchanged from legacy).

## Envelope / cleanup / audit confirmation

- All failure paths emit the human FAIL envelope + exit 1; summary
  artifact write failures WARN to stderr but never mask the run verdict.
- Handles: per-suite log fd closed by subprocess machinery; temp-file
  hygiene pinned by W5/W6.
- Audit rows: N/A for host tooling (spec §6, re-verified: no ledger or
  audit imports anywhere in the orchestration path).

## Verification

- Probes P1/P2 as above.
- Full CI through the hardened executor: **19/19 PASS, exit 0**
  (182902 ms wall) — no behavioral regression vs T-00116 baseline.
- `tools/test_ci_suites.py` W1..W7 green.
