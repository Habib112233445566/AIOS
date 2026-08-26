# T-00068 — Task Ledger Control automated tests: Hardening

**Date:** 2026-08-22
**Type:** hardening (suite robustness)
**Depends on:** T-00067 security review
**Honesty note:** written after `task done 68` (cwd slip on the original
write, same class as T-00066); content reflects exactly what shipped.

## Gaps found and fixed

1. **Unbounded subprocess waits.** Four ledger suites could hang CI
   forever if a binary ever deadlocked: `Sandbox.cli` (matrix), run()
   helpers in the CLI-wire and config suites, and the service suite's
   grant-mint call had no `timeout=`. All now carry explicit bounds
   (60–120 s, far above observed runtimes).
2. **Holder leak-safety.** The M5 lock-holder is killed in a `finally`
   if still alive after the assertion path (previously a mid-case
   failure could orphan it for its 6 s sleep — bounded but sloppy).

## Verification

All four touched suites re-run green:
```
PASS: task ledger matrix smoke (M1..M8)
PASS: task cli wire smoke (C1..C9)
PASS: task config smoke (K1..K5)
PASS: task service wire smoke (W1..W8)
```

## Acceptance check
- [x] Failure modes bounded (timeouts + kill-safety).
- [x] No leaks introduced; prior behavior unchanged.
