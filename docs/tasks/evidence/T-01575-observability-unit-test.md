# T-01575 — Filesystem Layout observability: Unit Test

## Metadata
- **Task ID:** `T-01575`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / observability
- **Status:** Complete — executed focused unit test suite for observability with 100% pass rate.
- **Date:** 2026-09-19
- **Depends on:** `T-01574` (Implementation)
- **Feeds:** `T-01576` (Integration)
- **Artifacts:** `docs/tasks/evidence/T-01575-observability-unit-test.md`, `docs/tasks/evidence/T-01575-unit-test.md`

---

## 1. Unit Test Scope & Results

Executed standalone test suite `code/aiosh-cli/tests/test_fs_layout_observability.py`:
- **O1 (Telemetry Emission Completeness)**: Both CLI and MCP invocations generate corresponding audit rows in SQLite WAL `audit.db`.
- **O2 (Audit Correlation & Queryability)**: `aiosh audit tail` accurately correlates generated audit events by target layout ID and tool name.
- **O3 (Outcome Fidelity)**: Asserts distinct logging for `"ok"`, `"error"`, and `"refused"` outcomes.
- **O4 (State Inspection Parity)**: Validates state parity (`active_layout` and `layouts` count) across CLI and MCP.
- **O5 (Destructive Mutation Flagging)**: Validates `destructive: true` flag on partition shrink deltas vs `destructive: false` on identical layouts.

---

## 2. Test Execution Output

```
=== RUNNING FILESYSTEM LAYOUT OBSERVABILITY TEST SUITE (FL12) ===
PASS: O1 Telemetry emission completeness verified for CLI and MCP
PASS: O2 Audit correlation & queryability verified by target and tool
PASS: O3 Outcome fidelity verified (ok, error, refused)
PASS: O4 State inspection parity verified (active=None, count=2)
PASS: O5 Destructive mutation flagging verified (true on shrink, false on identical)
PASS: All Observability criteria O1..O5 passed successfully.
```

---

## 3. Acceptance Confirmation

- [x] New test file runs standalone and passes.
- [x] Negative cases are asserted, not just happy path.
