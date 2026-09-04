# T-01177 — Base Image Build / Observability: Security Review

**Date:** 2026-09-04
**Type:** Security Review
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Observability

## 1. Threat Modeling & Abuse Scenarios
- **Scenario A: Memory Exhaustion via Unbounded Categorical Maps**:
  - Malicious inputs could inject thousands of distinct format or architecture labels to balloon map allocations.
  - Mitigation: Limit maximum map entries and validate string lengths in `validate()`.
- **Scenario B: Arithmetic Overflow in Aggregation**:
  - Accumulating large `size_budget_bytes` values could overflow 64-bit integers.
  - Mitigation: `saturating_add` prevents panic on sum overflow; `validate()` asserts arithmetic consistency.
- **Scenario C: Information Leakage**:
  - Verify that report outputs contain only operational telemetry (counts, sums, architectures, and formats) and do not expose sensitive files or raw image contents.
- **Scenario D: Audit Invariants & PEP Enforcement**:
  - Observability queries emit an audit row via `classify_and_emit` (CLI) and `dispatch::recorded_call` (MCP) into the hash-chained SQLite WAL audit ring.

## 2. Hardening Recommendations for T-01178
- Cap map entry counts for `format_breakdown`, `architecture_breakdown`, and `distro_breakdown`.
- Sanitize and cap kernel version strings to prevent oversized report serialization.
