# T-00052 — Task Ledger Control configuration: Specification

**Date:** 2026-08-22
**Type:** specification (no code changed)
**Depends on:** T-00051 research
**Status:** SPECIFIED — D1–D6 locked to research defaults (standing
autonomy, 2026-08-22). All new behavior is **AIOS-specific**; the
mechanism follows Twelve-Factor III (E1/E2, fetched 2026-08-22).

## 1. Resolved decisions

D1 knob set = the six operational knobs (F2). D2 names below. D3
invalid → loud error naming the variable, no silent fallback. D4 CLI
`aiosh task config` (audited). D5 **no MCP tool** — config is an
operator surface; agent-writable security knobs would be
anti-security. D6 Python mirror parity.

## 2. The `LedgerConfig` contract

| Env var | Default (= today) | Consumed by |
|---|---|---|
| `AIOSH_LEDGER_LOCK_TIMEOUT_SECS` | 5 | `acquire_lock` (rust+py) |
| `AIOSH_LEDGER_MAX_LEDGER_BYTES` | 67108864 (64 MiB) | `read_capped` |
| `AIOSH_LEDGER_MAX_EVENTS_BYTES` | 16777216 (16 MiB) | `read_capped` |
| `AIOSH_LEDGER_MAX_STATE_BYTES` | 4194304 (4 MiB) | `read_capped` |
| `AIOSH_LEDGER_MAX_TEXT` | 4096 | task_service validate + parse_args + MCP schema text |
| `AIOSH_LEDGER_MAX_EVIDENCE_ITEMS` | 16 | same |

Rules:
- Values are decimal integers; constraints: lock ≥1; byte caps ≥1024;
  text ≥64; evidence ≥1. Out-of-range/unparseable → error string
  `invalid AIOSH_LEDGER_<NAME>='<raw>': <why>` surfaced at first use
  (CLI: exit 1 envelope + audit row; MCP: `{ok:false}` envelope).
- Precedence: env var > built-in default. Nothing else (no files).
- Config is read at each operation start (cheap; no caching) so
  changes apply to the next command without restarts.
- Existing vars (AIOSH_TASKS_DIR/AIOSH_HOME/AIOSH_CONSTITUTION) are
  unchanged — already conformant (F1/E3 handles).

## 3. Interfaces

**New:** `aiosh-core/src/ledger_config.rs` (`LedgerConfig::from_env()`,
field getters; defaults module-level). `task_service` + `ledger`
consume it instead of consts (consts remain as the DEFAULT values).
`cmd_task` gains `config` subcommand (read-only print, audited like
every task subcommand; exit 0). Python `tools/task_ledger.py` +
`server.py` read the same six vars with identical defaults/constraints.

**Unchanged:** MCP tool set (D5), schemas, envelopes, grammar.

## 4. Failure matrix

| Condition | Result |
|---|---|
| unparseable / out-of-range var | error naming var at first use; CLI exit 1 + audit row; MCP ok:false |
| unknown `AIOSH_LEDGER_*` var set | ignored (forward compat), documented |
| `config` subcommand | prints `{ok:true, subcommand:"task config", data:{<knob>: {value, source}}}` |

## 5. Out of scope
File-based config (rejected, E2); MCP exposure (D5); other subsystems'
knobs (audit/pentest constants are separate components).

## 6. Reviewability check
Happy path §2; failures §4; reused/new §3 — reviewable standalone.
