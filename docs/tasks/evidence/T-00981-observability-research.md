# T-00981 — Agent Handoff Protocol / Observability: Research

## 1. Prior Art & Architecture
- Observability for handoffs builds upon `HandoffReport` and `HandoffStore::to_report()`.
- Observability metrics:
  - `total_handoffs`: Total historical count.
  - `active_handoffs`: Sum of `Pending` + `Accepted` in-flight handoffs.
  - `completed_handoffs`: Count of successfully completed handoffs.
  - `rejected_handoffs`: Count of rejected handoffs.
  - `cancelled_handoffs`: Count of cancelled handoffs.
  - Latency/duration calculation from `created_at` timestamp.
- CLI exposure: `aiosh handoff stats` / `aiosh handoff report`.
- Invariant testing: Criterion `H8` in `tools/test_handoff_suites.py`.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Metrics Location | Fact | In `HandoffReport` within `code/aiosh-rust/aiosh-core/src/handoff.rs`. |
| Aggregate Invariant | Fact | `active + completed + rejected + cancelled == total_handoffs`. |
| Audit Row Invariant | Fact | Stats queries are read-only and emit zero modifying audit rows. |
