# T-00059 — Task Ledger Control configuration: Documentation

**Date:** 2026-08-22
**Type:** documentation (no code changed)
**Depends on:** T-00058 hardening

## What shipped

1. **`docs/SPEC-TASK-LEDGER.md` §8.3 (new)** — configuration reference:
   six-knob table (variable/default/floor/ceiling), precedence rules,
   loud-error contract, no-MCP-exposure decision, and a live-verified
   copy-pasteable example (`task config` output captured verbatim with
   one env override active).
2. **§9 evidence index** extended with rows T-00040..T-00059.

## Acceptance check
- [x] Spec updated with what shipped + invocation.
- [x] Copy-pasteable example (live-verified, override visible).
- [x] Constraints stated (floors/ceiling, schema note, unknown-var policy).
- [x] Evidence files linked.
