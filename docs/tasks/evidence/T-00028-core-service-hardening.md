# T-00028 — Task Ledger Control core service: Hardening

**Date:** 2026-08-22
**Type:** hardening (behavior change: bounded lock wait; transport cap)
**Depends on:** T-00027 security review

## Gaps hardened (found by re-auditing the service path, not assumed)

1. **Unbounded lock wait → bounded, loud failure.**
   `flock(LOCK_EX)` blocked *forever* if a writer ever hung while
   holding `.TASK_STATE.lock` — a stuck CLI would freeze the MCP
   server's task tool indefinitely with no error and no audit row.
   Now (Rust `ledger::acquire_lock_timeout` + Python
   `task_ledger.acquire_lock`, identical contract): poll
   `LOCK_EX|LOCK_NB` every 50 ms up to `LOCK_TIMEOUT_SECS = 5`, then
   fail with an explicit error —
   `ledger lock busy after 5000ms (another writer holds .TASK_STATE.lock?)` —
   which flows into the standard envelope and the honest audit row.
   Non-EWOULDBLOCK flock errors still fail immediately.

2. **Unbounded stdin line → hard transport cap.**
   The MCP loop read lines with no size limit; a hostile client could
   balloon server memory with one giant JSON line. Now
   `read_line_capped` bounds lines at `MAX_LINE_BYTES = 1 MiB`
   (largest legitimate request ≈ 70 KiB: 4096-note + 16×4096
   evidence). Over-cap lines are drained to the newline (framing for
   subsequent requests preserved) and answered with JSON-RPC
   `-32700 "request line exceeds 1048576 bytes"`.

3. **Verified-not-added:** error-envelope consistency (every failure is
   `ok:false`/`isError` or a protocol error — no silent paths),
   resource cleanup (no temp/connection/child leaks on error paths;
   state tmp cleanup from T-00018 unchanged and still covered by
   tests), size caps (schema + file caps from T-00018/T-00025).

## Tests added

- Rust `ledger::tests::lock_contention_times_out_with_explicit_error` —
  holds the lock via a second open-file description (same-process
  contention), asserts the busy-error text, ≥140 ms wait, and success
  after release. (64 cargo tests total, zero warnings.)
- Python `U16`/`U16b` in `tools/test_task_ledger.py` — same contention
  scenario through the module surface (0.20 s observed), then normal
  mutation after release. Suite now U1..U16.

## Live verification

```
oversized line (1 MiB + 10 bytes) → {"error":{"code":-32700,
  "message":"request line exceeds 1048576 bytes"}}
next request on the same connection → ok:true   # framing preserved
$ bash ci/run_all_smokes.sh → == ALL 11 SMOKE SUITES PASS ==
```

## Behavior-change note (for T-00029 docs)

Two concurrent writers now deterministically produce one success and
one explicit `lock busy` error after 5 s — previously the second
silently waited forever. `docs/SPEC-TASK-LEDGER.md` §7 L1 wording and
the operator guide must be refreshed accordingly in the Documentation
task.
