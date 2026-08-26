# T-00018 — Task Ledger Control data model: Hardening

**Completed:** 2026-08-21

## Acceptance criteria
- [x] Failure modes produce explicit, auditable errors.
- [x] No temp/connection leaks on the error path.

## 1. Size caps (untrusted file reads)

`code/aiosh-rust/aiosh-core/src/ledger.rs` — `read_capped()` bounds every
untrusted file read before loading:

| File | Cap |
|---|---|
| `MASTER_TASK_LEDGER.jsonl` | 64 MiB (10k tasks ≈ 200 KB typical) |
| `COMPLETIONS.jsonl` | 16 MiB |
| `TASK_STATE.json` | 4 MiB |

A file over the cap fails with an explicit error, e.g.:
```
COMPLETIONS.jsonl too large (17829502 bytes > cap 16777216 bytes)
```
Verified empirically: corrupting the event log past the cap makes
`aiosh task rebuild` refuse loudly instead of exhausting memory.

## 2. Temp-file cleanup (no leaks on the error path)

- `save_state_atomic` now runs `cleanup_stale_tmp()` first — any
  `<state>.tmp.<pid>` leftover from a crashed/interrupted writer is
  removed so `create_new` (O_EXCL) can never collide with a dead pid.
- On a write/fsync failure, the function removes its own temp file
  before returning the error — no orphan `.tmp` on the error path.
- Verified empirically: two planted `TASK_STATE.json.tmp.<pid>` files
  were removed by the next save; the live state file is untouched.
- RAII throughout: `FileLock` unlocks on drop; file handles close on
  scope exit. No DB connections or child processes exist in the ledger
  path.

## 3. Standard error envelope (never silent failure)

All failures return `{"ok": false, "error": <reason>}` with non-zero
exit — no partial state writes (atomic rename only on the success path),
no swallowed errors.

## 4. Fail-open audit honesty (ADR-0035 §F-2)

**Wired `aiosh task` through the SQLite audit ring** (previously it only
wrote the ledger's own event log). Every `aiosh task` invocation now
emits exactly one audit row — including refusals and errors:

```
row: task.ledger | ok      | aiosh task done 18 --note hardening-test
row: task.ledger | refused | NO-SKIP violation: attempted to complete T-00005...
```

Verified empirically: `aiosh audit tail` shows the success row and the
no-skip refusal row (with the honest reason), so no state-changing
action is ever silent.

## Tests

2 new Rust unit tests (52 total, zero-warning build):
`stale_tmp_files_are_cleaned_on_save` and
`events_size_cap_rejects_oversized_log`. Full `ci/run_all_smokes.sh`
green (10/10).

## Constraints / limitations

- Timeouts and bounded retries are not applicable: the ledger performs no
  external-process or network operations (file-only data model); size
  caps bound the file I/O instead.
- Caps are generous (64/16/4 MiB) and documented; they are a DoS guard,
  not a policy limit.
