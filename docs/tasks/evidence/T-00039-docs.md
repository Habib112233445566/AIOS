# T-00039 — Task Ledger Control CLI surface: Documentation

**Date:** 2026-08-22
**Type:** documentation (no code changed)
**Depends on:** T-00038 hardening

## What shipped

1. **`docs/SPEC-TASK-LEDGER.md` §8.1 (new)** — CLI unified-validation
   reference: the three behavior changes vs the pre-T-00034 CLI
   (empty-note refusal, caps, dash-value/`--` rule), live-verified
   copy-pasteable commands (`task help`, `task status`, `done --note`,
   a real refusal snippet), stream convention, and the T-00038
   non-UTF-8 lossy guarantee.
2. **§9 evidence index** extended with rows T-00030..T-00039.

## Verification

Both embedded examples executed immediately before writing:
`aiosh task help` (usage text captured) and `aiosh task done 1 --note ""`
(refusal JSON captured verbatim).

## Acceptance check

- [x] Spec updated with what shipped + how to invoke.
- [x] Copy-pasteable example included (live-verified).
- [x] Constraints stated honestly (behavior-change list; G9 deviation).
- [x] Evidence files linked (§9 index rows).
