# T-00095 — documentation component: Unit Test (evidence)

**Date:** 2026-08-22 · **Suite:** `tools/test_task_docs.py` (new, U1–U18)
**Result:** **18/18 PASS standalone**; production tree untouched
(verified: real SPEC intact, `check_task_docs.py` still C1–C6 green).

## Method

Fixture-driven behavioral tests: each case builds an isolated temp
docs-tree and injects it by rebinding the checker module's artifact
attributes (`ROOT/SPEC/INDEX_MD/LEDGER_JSONL/GOALS/DOCS_README`),
restored in `finally`. No production signatures changed; assertions
target observable `(ok, detail)` outputs, never internals.

## Coverage matrix

| Cases | Behavior |
|---|---|
| U01–U03 | C1 valid / missing file / TODO marker |
| U04–U06 | C2 all-six-sections ok / heading absent / frozen-range mismatch |
| U07–U10 | C3 fenced path ignored / missing path flagged / placeholder ignored + unterminated-fence boundary |
| U11–U12 | C4 consistent mini phase-map ok / range mismatch failure |
| U13–U15 | C5 clean index ok / TODO in GOALS / broken relative link |
| U16–U17 | C6 no-counts ok / volatile `CI n/n` flagged (file:line in detail) |
| S1 | **Broken-feature proof:** with C6's regex blinded via injection, the hostile fixture passes silently — demonstrating the suite detects a blind checker (the primary failure mode). Regex restored immediately. |

## Broken-feature proof (live, this session)

- Pre-fix red states observed while writing the suite: U17 initially
  asserted on matched-text instead of the checker's actual
  `path:line` detail contract → assertion corrected to the real
  observable output (checker behavior was correct; test fixed).
- S1 mechanically proves detection-of-blindness end-to-end.

## Test-side defects caught during authoring (honesty record)

1. Sandbox initially bound plain strings where the checker expects
   `Path` objects (AttributeError) — bound `Path`.
2. U02's first draft removed files via the SAVED production attribute —
   would have touched the real SPEC. Fixed to remove only the fixture
   path (`cd.SPEC` post-bind); production SPEC byte-size verified
   unchanged after the full run.
