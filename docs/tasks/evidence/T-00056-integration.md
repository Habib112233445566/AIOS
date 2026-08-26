# T-00056 — Task Ledger Control configuration: Integration

**Date:** 2026-08-22
**Type:** integration wiring
**Depends on:** T-00055 unit tests

## What shipped

- **CI registration** — `ci/run_all_smokes.sh` gains
  `task_config_smoke`, placed after `task_cli_smoke`. The env-config
  contract (defaults, overrides applied end-to-end, loud named errors,
  floors) is now exercised on every baseline run.
- **Discoverability** — `aiosh task config` listed in the task help
  overview (shipped in T-00054); suite K1 pins the output shape.
- **Cross-substrate parity** — Python mirror reads the same six vars
  with identical defaults/floors (`_env_float`/`_env_int`); parity is
  asserted by construction + spot probe in T-00054 evidence. File-level
  cross-substrate flows remain green in rust_smoke.

## Verification

```
$ bash ci/run_all_smokes.sh → == ALL 14 SMOKE SUITES PASS ==
PASS: task_config_smoke     # NEW
```

## Acceptance check
- [x] Feature reachable through production surface (CLI subcommand +
      env layer), end-to-end in CI.
- [x] Registration/discoverability updated.
- [x] Integrated-path smokes pass.
