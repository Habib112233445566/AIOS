# T-00901 — Regression Triage / Recovery & Validation: Research

## 1. Prior Art & Subsystem Recovery Patterns
- **Store Recovery & Resilience**:
  - `TriageStore` recovery from corrupted/malformed JSON files on disk without unrecoverable panic.
  - Fail-safe fallback to clean in-memory state with honest error diagnostics.
- **Validation Engine**:
  - `validate_triage_record`: Asserts 64-character SHA-256 signature, non-empty `test_target` and `error_message`, and valid RFC-3339 timestamps.
  - `validate_triage_report`: Mathematical consistency between `open_records + resolved_records == total_records`.
- **Blocker Health Gating**:
  - `has_unresolved_blockers()` helper for CI release gates.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Corrupt File Handling | Fact | Corrupted JSON store files must be handled gracefully without panic. |
| Invariant Validation | Fact | Invariants must be enforced on records prior to store insertion and report emission. |
| Blocker Verification | Fact | CI gate checks must accurately distinguish blocker/critical from major/minor. |

## 3. Decisions & Actions
- Implement `validate_triage_record` and store recovery helper in `aiosh-core::triage`.
- Add criterion `T8` (recovery & validation) to `tools/test_triage_suites.py`.
