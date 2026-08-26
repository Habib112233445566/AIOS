# T-00036 — Task Ledger Control CLI surface: Integration

**Date:** 2026-08-22
**Type:** integration wiring
**Depends on:** T-00035 unit tests

## What shipped

- **CI registration** — `ci/run_all_smokes.sh` gains `task_cli_smoke`
  (`code/aiosh-cli/tests/test_task_cli_smoke.py`), placed immediately
  after `cli_bash_smoke` so the binary is guaranteed built (rust_smoke
  runs first). The unified CLI validation is now exercised on every
  baseline run through its production surface (real binary, real
  files, real audit ring).
- **Cross-substrate parity** — unchanged and still green: the ledger
  files the CLI writes are the same ones rust_smoke's 4-flow
  Rust↔Python parity step exercises (done/block/unblock+skip/rebuild
  replay). CLI-side validation unification does not touch the shared
  state format (verified: full CI below).
- **Discoverability** — `aiosh task help` (per-subcommand usage) and
  the corrected top-level help line shipped in T-00034; the suite
  asserts help exit-0/no-side-effects (C9).

## Verification

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke … PASS: cli_bash_smoke
PASS: task_cli_smoke        # NEW in CI
PASS: task_ledger_unit … PASS: task_ledger_scaffold
== ALL 12 SMOKE SUITES PASS ==
```

## Acceptance check

- [x] Feature reachable through its production surface, end-to-end in CI.
- [x] Cross-substrate parity confirmed (4-flow step green in same run).
- [x] Registration/discoverability updated (CI runner + help surface).
- [x] Closest existing smokes for the integrated path pass.
