# T-00811 — Regression Triage / data model: Research

## 1. Prior Art & Subsystem Objectives
- **Context & Goal**:
  - `Regression Triage (T-00811..T-00910)` establishes an automated tracking and categorization subsystem for test failures, broken invariants, and flaky regressions within the AIOS userspace ecosystem.
  - Works alongside CI summary data structures (`aiosh-core::ci::RunSummary`) to ingest test failure records and correlate them into deduplicated failure signatures.
- **Data Model Requirements**:
  - `TriageStatus`: Enum tracking lifecycle state (`Untriaged`, `Triaged`, `FixPending`, `Resolved`, `WontFix`).
  - `TriageSeverity`: Discrete impact priority (`Blocker` / P0, `Critical` / P1, `Major` / P2, `Minor` / P3).
  - `TriageRecord`: Granular record containing deduplication `signature` (SHA-256 hash of normalized error message), `test_target`, `error_message`, `repro_command`, occurrences counter, lifecycle timestamps, and optional blame annotations (`blame_task_id`, `blame_commit`).
  - `TriageReport`: Aggregated report tracking statistics and active triage issues.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Fingerprinting | Fact | Deduplication signatures should use SHA-256 digests over normalized failure messages to group repeated test failures. |
| Audit Compliance | Fact | State mutations (recording a new triage item, updating resolution) must be non-destructive and auditable. |
| Memory Limits | Fact | Serialized triage entries and reports must enforce bounded string lengths and memory caps. |

## 3. Decisions & Contracts Needed
1. Define `TriageStatus`, `TriageSeverity`, `TriageRecord`, and `TriageReport` in `code/aiosh-rust/aiosh-core/src/triage.rs`.
2. Provide deterministic signature generation helper `compute_failure_signature(test_target: &str, error_msg: &str) -> String`.
3. Register `pub mod triage;` in `aiosh-core/src/lib.rs`.
