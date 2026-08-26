# T-00033 — Task Ledger Control CLI surface: Scaffold

**Date:** 2026-08-22
**Type:** scaffold (interfaces only; production path untouched)
**Depends on:** T-00032 spec

## What shipped

In `code/aiosh-rust/aiosh-cli/src/main.rs`, immediately beside the
legacy `flag_after` helper it will replace:

| Interface | Kind | Spec anchor |
|---|---|---|
| `parse_task_args(&[String]) -> Result<TaskArgsOwned, String>` | fn (todo!) | spec §2.1 — argv mirror of `task_service::parse_args`: u64≥1 ids, non-empty note/reason, 4096 cap, ≤16 evidence, dash-leading values rejected, `--` delimiter |
| `task_usage_text(Option<&str>) -> String` | fn (todo!) | spec D3 — per-subcommand usage/help table |

Both carry `#[allow(dead_code)]` with an explicit
`TODO(T-00034): remove this allow` marker (bin-crate private fns would
otherwise trip dead_code before their production call sites exist).
Doc comment on the block records the full contract and the research
defects (Q2/Q3/Q4/Q6) being eliminated.

## Deliberately NOT touched

`cmd_task` still uses the permissive legacy parsing this task — wiring
the new parser into the production path is exactly the T-00034
Implementation step. Verified: `aiosh task status` works and
`cli_bash_smoke` PASSes with the scaffold in tree.

## Verification

```
$ cargo build      → 0 errors, 0 warnings
$ cargo test -p aiosh-cli
  test task_cli_scaffold_tests::scaffold_bodies_are_unimplemented - should panic ... ok
  test task_cli_scaffold_tests::scaffold_signatures_compose ... ok
  test result: ok. 2 passed
$ cargo test (workspace) → 64 passed; 0 failed
$ bash code/aiosh-cli/tests/smoke.sh → PASS (prod path regression-free)
```

## Acceptance check

- [x] Project builds with zero errors (and zero warnings).
- [x] New interfaces exist and are referenced by test stubs
      (should-panic call + fn-pointer composition proof).
- [x] Bodies fail loudly (`todo!` → "not yet implemented"), asserted.
- [x] Existing behavior regression-free (CLI status + bash smoke green).

## Handoff to T-00034

Implement both bodies per spec §2/§2.1 (incl. `--` delimiter,
dash-value rejection, caps, `help`), swap `cmd_task` onto
`parse_task_args`, delete both `#[allow(dead_code)]` markers, add
positive/negative CLI tests (empty note now refused, caps, `--`
passthrough), keep zero warnings, full CI green.
