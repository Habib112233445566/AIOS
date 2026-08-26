# T-00054 — Task Ledger Control configuration: Implementation

**Date:** 2026-08-22
**Type:** implementation (Rust + Python parity)
**Depends on:** T-00053 scaffold; spec T-00052

## What shipped

- **`ledger_config.rs` implemented**: `from_env()` (env > default;
  loud `invalid AIOSH_LEDGER_<NAME>='<raw>': <why>` errors; range
  floors 1/1024/64) + `to_json_with_sources()`; dependency-injected
  `from_source`/`to_json_with_sources_from` variants so tests avoid
  process-env mutation entirely (first attempt used env mutation and
  raced parallel tests — refactored away, documented honestly).
- **Consumers switched**: `ledger.rs` file caps + lock timeout;
  `task_service` text/evidence caps (consts remain as DEFAULTS; the
  published MCP inputSchema stays at default values — env effectively
  tightens wire clients, documented in code).
- **MCP validation-gap fix**: `call_task` never invoked `validate()`
  (conditional presence/caps) — `done` without note would have stored
  empty. Now `call.validate()?` runs before execute (single source).
- **CLI `aiosh task config`** — audited read-only print of effective
  values + per-knob `source` (env|default); listed in help.
- **Python parity**: `task_ledger.py` lock timeout and `server.py`
  text/evidence caps read the same six variables with identical
  defaults/constraints via `_env_float`/`_env_int` (loud SystemExit).

## Verification (live)

```
$ aiosh task config                     → 6 knobs, source=default
$ AIOSH_LEDGER_MAX_TEXT=8192 … config   → max_text {value:8192, source:env}
$ AIOSH_LEDGER_MAX_TEXT=20 … done --note(30 chars)
  → refused "invalid AIOSH_LEDGER_MAX_TEXT='20': must be >= 64"
$ AIOSH_LEDGER_MAX_TEXT=8192 … done --note(30) → accepted
$ AIOSH_LEDGER_LOCK_TIMEOUT_SECS=soon … status
  → "invalid AIOSH_LEDGER_LOCK_TIMEOUT_SECS='soon'" (audited refusal)
$ python mirror: AIOSH_LEDGER_MAX_TEXT=64 → MAX_TASK_TEXT == 64
$ cargo test → 79 passed; 0 failed (0 warnings)
```

## Acceptance check
- [x] Targeted tests pass (from_source precedence/errors/sources).
- [x] No regression (build zero-warning; suites re-run in T-00056).
- [x] No new dependencies.
