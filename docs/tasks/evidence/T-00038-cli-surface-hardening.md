# T-00038 — Task Ledger Control CLI surface: Hardening

**Date:** 2026-08-22
**Type:** hardening (one real panic-path fix + cleanup audit)
**Depends on:** T-00037 security review

## Gap found and fixed (proven empirically, before and after)

**Non-UTF-8 argv panicked the whole binary.**
`std::env::args()` panics on invalid unicode — verified BEFORE the fix:
hostile argv (`task status \xff\xfe`) → **exit 101 with a Rust panic**,
no envelope, no audit row. Fixed in `main()` via
`args_os().map(to_string_lossy)`: the same invocation now yields
exit 1 + the standard refusal envelope + one honest `task.ledger` row
whose args preserve the replacement chars for forensics
(`['\ufffd\ufffd']` observed). This protects EVERY subcommand, not just
`task`.

## Verified-not-added (re-audited per template)

- **Timeouts:** lock wait bounded at 5 s since T-00028 (shared path);
  no external processes are spawned by the task CLI path.
- **Size caps:** schema caps enforced at parse/validate (T-00034);
  file caps from T-00018; OS ARG_MAX bounds argv.
- **Standard envelope:** every failure — usage, semantic, runtime —
  exits 1 through `err_out`; success through `ok_out`. No silent paths.
- **Resource cleanup:** task path creates no temp files or child
  processes; state tmp cleanup covered by existing tests
  (`stale_tmp_files_are_cleaned_on_save`, U11); lock file persists by
  design (it IS the lock).

## Verification

```
before: returncode 101, panic thread 'main' (env.rs:878)
after : returncode 1, {"ok":false,"error":"unexpected argument '\ufffd\ufffd'…"}
        audit row args = ['\ufffd\ufffd']   ← lossy bytes preserved
$ cargo build → 0 warnings ; $ cargo test → 77 passed; 0 failed
```

## Acceptance check

- [x] Failure modes produce explicit, auditable errors (incl. the new
      non-UTF-8 path).
- [x] No temp/connection leaks on the error path (re-audited).
