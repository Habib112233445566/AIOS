# T-00103 — Task Ledger Control / recovery & validation: Scaffold

Date: 2026-08-23 · Status: SCAFFOLD COMPLETE

## What was added (interfaces only, bodies fail loudly)

| Substrate | Artifact | Call site |
|---|---|---|
| Python reference | `tools/task_ledger.py::validate_state(state_path, events_path, ledger_path) -> dict` raising `NotImplementedError("validate_state is scaffolded (T-00103); implementation lands in T-00104")` | wired into `main()` argparse: `sub.add_parser("validate", ...)` → `result = validate_state()` |
| Rust ship path | `aiosh-core/src/ledger.rs::pub fn validate_state(p: &LedgerPaths) -> Result<Value, String>` with `unimplemented!()` | referenced by unit test `ledger::tests::validate_state_scaffold_fails_loudly` (`#[should_panic(expected = "scaffolded (T-00103)")]`) |

No MCP manifest change yet (enum extension is T-00106 integration scope, per
spec §7). No new dependencies. No behavior change to any existing path.

## Build/import verification (commands + results)

- `python3 -c "import task_ledger; task_ledger.validate_state()"` →
  import ok, loud `NotImplementedError` raised (fail-loudly contract).
- `python3 tools/task_ledger.py validate` → loud traceback naming the
  scaffold (never a silent success); exit code surfaced by shell wrapper.
- `cargo build` (workspace) → `Finished dev profile` — **zero warnings**.
- `cargo test --lib validate` → 2 passed incl. the should_panic scaffold pin.
- Regression: `python3 tools/test_task_ledger.py` → **PASS U1..U16**.

## Acceptance mapping

- Project builds/imports with zero errors ✅ (both substrates verified above).
- New interfaces exist and are referenced by ≥1 call site or test stub ✅
  (Python CLI subcommand + Rust should_panic test).

## Notes

- A splicing error during the first edit briefly duplicated
  `_count_ledger_lines` and truncated `find_task_in_ledger`; caught by
  immediate re-read of the region and fixed before any test run. Final tree
  contains exactly one definition of each function.
