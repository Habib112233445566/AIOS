# T-00034 — Task Ledger Control CLI surface: Implementation

**Date:** 2026-08-22
**Type:** implementation (Rust, aiosh-cli + one shared core cap)
**Depends on:** T-00033 scaffold; spec T-00032 (D1–D5 owner-locked)

## Failing-tests-first record

13 behavioral tests written before/with the implementation; they caught
two real defects during development:

1. **`--` in value position**: `--note -- -flag` was rejected because
   the delimiter check ran before value capture. Fixed: a bare `--`
   while expecting a value becomes the delimiter and the FOLLOWING
   token is taken as the value.
2. **Off-by-one after `--evidence` values**: `take_value` leaves the
   cursor on the value token; the arm's `continue` skipped the advance,
   re-parsing `"a.md"` as an unexpected bare token. Fixed with explicit
   `i += 1`.

Also tightened by tests: caps live in `TaskCall::validate` (single
source with MCP) — oversized text parses then fails validation;
`status 5` refuses at parse ("unexpected argument") which is strictly
earlier than the spec minimum.

## What shipped

- **`parse_task_args(argv)`** (`aiosh-cli/src/main.rs`) — strict argv →
  `TaskArgsOwned`: decimal u64 ≥ 1 single operand where required;
  `--note/--reason/--evidence` with non-optional values; values may not
  start with `--` unless after a lone `--` (G7/G14/G10); unknown
  dash-tokens named in errors; evidence ≤16 items; per-subcommand
  usage text embedded in every refusal.
- **`task_usage_text(Option<sub>)`** — overview + 8 detailed pages
  (incl. `help`), documenting the no-skip law, caps, delimiter rule,
  and the intentional POSIX-G9 deviation (D5).
- **`cmd_task` rewired** onto `parse_task_args → TaskCall::validate →
  execute_with(&paths)` — ONE validation source shared with MCP (D1).
  Empty-note refusals now identical across surfaces. Envelope label
  fixed (`"task"` not `"task "`); `help` exits 0 without audit row;
  every other outcome still writes exactly one honest audit row.
- **Core cap completion** (`aiosh-core/task_service.rs`): evidence ITEM
  length ≤4096 enforced in both `validate()` and `parse_args()` (gap
  found while unifying; previously only count was capped).
- Removed legacy `flag_after`; scaffold `allow(dead_code)` markers gone.

## Verification

```
$ cargo build    → 0 warnings
$ cargo test     → 77 passed; 0 failed   (64 core incl. new evidence-cap
                                          tests + 13 CLI behavioral)
$ live probes (scratch ledger):
  done --note ""            → refused "'note' must be non-empty…"
  block --reason --force    → refused "must not start with \"--\""
  done … --note -- -x       → note stored "-x"        (-- passthrough)
  skip --reason -- --weird  → reason "--weird" verbatim in state+event
  task help                 → exit 0
$ bash ci/run_all_smokes.sh → == ALL 11 SMOKE SUITES PASS ==
```

## Acceptance check

- [x] Targeted tests pass (13 new CLI unit tests).
- [x] No regression: full baseline CI 11/11 PASS.
- [x] No new dependencies; audit/PEP invariants intact.
