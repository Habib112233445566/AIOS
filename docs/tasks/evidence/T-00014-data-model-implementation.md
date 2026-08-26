# T-00014 — Task Ledger Control data model: Implementation

> **Amendment (2026-08-21): Rust port.** The data model is now
> implemented **in Rust** (`code/aiosh-rust/aiosh-core/src/ledger.rs`,
> port of `tools/task_ledger.py` with identical semantics: atomic state
> pointer via tmp+rename, append-only fsync'd event log, flock
> single-writer lock, no-skip enforcement, block/unblock/skip,
> rebuild-from-events, invariant check). It is reachable through the
> production CLI as `aiosh task <status|done|block|unblock|skip|rebuild|check>`
> (T-00016 integration). Cross-substrate parity is proven both ways:
> Python reads Rust-written state/events, Rust reads Python-written
> state/events (asserted by the `rust_smoke` CI parity step). The
> Python `tools/task_ledger.py` remains as the legacy reference and
> test oracle.

**Completed:** 2026-08-21

## Acceptance criteria
- [x] Targeted test passes.
- [x] No regression in existing smoke suites for touched modules.

## What was implemented

`tools/task_ledger.py` — full implementation of the spec in
`docs/tasks/evidence/T-00012-data-model-specification.md`:

- `load_state` / `save_state_atomic` — v1→v2 migration; atomic tmp-file +
  `os.replace` + fsync writes of `TASK_STATE.json`.
- `append_event` / `read_events` — append-only `COMPLETIONS.jsonl` event
  log with monotonic `seq`, fsynced per event.
- `rebuild_state` — recomputes the pointer from the event log
  (complete rebuild).
- `find_task_in_ledger` / `assert_ledger_invariants` — ledger lookup and
  structural validation (contiguous ids, `depends_on == [id-1]`,
  `next_task == id+1`).
- `acquire_lock` — `fcntl.flock` exclusive advisory lock.
- `complete_task`, `block_task`, `unblock_task`, `skip_task` — mechanical
  no-skip enforcement; every transition appends exactly one event.
- CLI subcommands `done/block/unblock/skip/rebuild/check/status` plus
  legacy `complete_task.py <id> [--note S]` compatibility.

`tools/complete_task.py` is now a thin wrapper delegating to
`task_ledger.main`, preserving the documented CLI contract.

## Verification

Sandboxed functional test (temporary state/ledger files; real ledger
untouched):

```
PASS v1->v2 migration
PASS complete 1 -> next 2, event seq 1, evidence stub
PASS no-skip refusal
PASS block/unblock
PASS skip -> next 3
PASS complete last -> next None
PASS rebuild from event log
PASS tampered ledger detected: next_task 3 != 2
PASS no leftover tmp files
```

`tools/test_task_ledger_scaffold.py` (interface check) PASS.
Full CI suite `bash ci/run_all_smokes.sh` re-run: all 7 suites PASS
(no regression in touched modules).
