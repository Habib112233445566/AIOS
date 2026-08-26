# T-00053 — Task Ledger Control configuration: Scaffold

**Date:** 2026-08-22
**Type:** scaffold (interfaces only; bodies fail loudly)
**Depends on:** T-00052 spec

## What shipped

New `code/aiosh-rust/aiosh-core/src/ledger_config.rs`, wired via
`lib.rs`:

- `LedgerConfig` struct (six knobs, `Default` == today's shipped
  constants — 5 s lock, 64/16/4 MiB caps, 4096 text, 16 evidence).
- `LedgerConfig::from_env()` — todo! body (T-00054).
- `to_json_with_sources()` — todo! body; `{"value":n,"source":"env"|"default"}`.
- `defaults_json()` — implemented (pure default projection).

Python mirror + consumers (`ledger.rs`/`task_service.rs`/`cmd_task`)
are T-00054 scope; production behavior untouched this task.

## Verification
```
$ cargo build  → 0 warnings
$ cargo test -p aiosh-core ledger_config → 2 passed (should-panic + defaults proof)
```

## Acceptance check
- [x] Builds clean; interfaces referenced by test stubs; loud bodies.
