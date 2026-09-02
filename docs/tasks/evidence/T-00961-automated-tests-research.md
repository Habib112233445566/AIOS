# T-00961 — Agent Handoff Protocol / Automated Tests: Research

## 1. Prior Art & Architecture
- Automated testing architecture matches other AIOS subsystems (`ci`, `task`, `doc`, `evidence`, `repo`, `secrets`, `triage`).
- Test dimensions:
  - **Data model boundary fuzzing**: Large payloads, unicode context strings, invalid ID formats.
  - **State transition matrix**: All combinations of valid and invalid transitions (`Pending -> Accepted -> Completed`, `Pending -> Rejected`, `Pending -> Cancelled`, `Accepted -> Cancelled`, terminal immutability).
  - **Signature determinism**: Ensuring canonical JSON formatting guarantees identical SHA-256 for identical handoffs.
  - **Storage and recovery**: Verifying corruption recovery and size cap enforcement.
- Integrated into `tools/test_handoff_suites.py` as Criterion `H6`.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Runner Location | Fact | `tools/test_handoff_suites.py` and `tools/test_handoff_unit.py`. |
| Edge Test Suite | Fact | Crate level unit tests in `code/aiosh-rust/aiosh-core/src/handoff_service.rs` and standalone runner. |
| Invariant Verification | Fact | 100% PASS with 0 return codes across all criteria H1..H6. |
