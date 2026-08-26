# T-00104 — Task Ledger Control / recovery & validation: Implementation

Date: 2026-08-23 · Status: IMPLEMENTATION COMPLETE (Python + Rust)

## What shipped

`validate_state` — the read-only integrity report defined by
`docs/tasks/evidence/T-00102-spec.md`:

| Substrate | Change |
|---|---|
| Python `tools/task_ledger.py` | replay loop factored into `_replay_events()` (shared with `rebuild_state` — zero duplicated semantics); `validate_state()` implemented: G1 drift / G2 seq integrity / G5 pointer range / G3+G4 evidence findings per spec §4 |
| Rust `aiosh-core/src/ledger.rs` | identical refactor into `replay_events()`; `validate_state(&LedgerPaths)` implemented with the same stable findings key set; scaffold `unimplemented!()` removed |

Reused existing helpers only (`load_state`, `read_events`,
`count_ledger_lines`, evidence-dir convention). **No new dependencies.**
No audit/PEP invariants touched: validate performs zero writes; surface
wiring (gate + one honest audit row) is T-00106 integration scope.

## Test-first note

The T-00103 should_panic scaffold pin was superseded by behavior tests
during this task:
- `validate_state_clean_repo_is_consistent` — happy path, key-set shape.
- `validate_state_detects_drift_without_mutating` — seeded hand-edit of
  `next_task`; asserts fatal + field list AND byte-identical state file
  after the run (report-only proof).
- `validate_state_detects_seq_gap` — first attempt (truncate+rebuild) was
  self-defeating and FAILED as designed-by-mistake; rewritten to renumber
  an event's `seq` (replay-equivalent) so ONLY the integrity check fires.

## Verification (commands + results)

- `cargo build` → Finished, **zero warnings**.
- `cargo test` → **13 + 69 = 82 passed, 0 failed**.
- `python3 tools/test_task_ledger.py` → **PASS U1..U16** (refactor of
  `rebuild_state` did not move any pinned behavior).
- Live repo probe: `python3 tools/task_ledger.py validate` →
  `consistent:true`, all structural checks ok, `evidence` warning listing
  true positives from legacy rows (attested `T-000NN` vs artifact
  `T-000NN.md`) — exactly the L4 gap the spec targets, reported not healed.
- `bash code/aiosh-rust/ci/rust_smoke.sh` → ALL PASS incl. MCP wire
  contract + 3 cross-substrate ledger parity legs (no regression).

## Acceptance mapping

- Targeted tests pass ✅ (three new Rust behavior tests; Python verified
  against live data).
- No regression in existing smoke suites for touched modules ✅
  (U1..U16, cargo suite, rust_smoke parity legs).
