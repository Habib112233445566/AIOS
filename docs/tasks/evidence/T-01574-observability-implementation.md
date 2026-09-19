# T-01574 — Filesystem Layout observability: Implementation

## Metadata
- **Task ID:** `T-01574`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / observability
- **Status:** Complete — implemented observability test suite `code/aiosh-cli/tests/test_fs_layout_observability.py` exercising O1..O5.
- **Date:** 2026-09-19
- **Depends on:** `T-01573` (Scaffold)
- **Feeds:** `T-01575` (Unit Test)
- **Artifacts:** `docs/tasks/evidence/T-01574-observability-implementation.md`, `docs/tasks/evidence/T-01574-implementation.md`

---

## 1. Implementation Details

Implemented the observability test cases in `code/aiosh-cli/tests/test_fs_layout_observability.py`:
- **O1 (Telemetry Emission Completeness)**:
  - Executes CLI `aiosh layout validate --standard --json` and MCP `aios.fs_layout.get`.
  - Asserts both invocations record entries in SQLite WAL `audit.db` with correct tools and commands.
- **O2 (Audit Correlation & Queryability)**:
  - Registers a layout with target ID `o2-obs-layout-v1`.
  - Correlates the generated `audit_id` with records retrieved via `aiosh audit tail`.
- **O3 (Outcome Fidelity)**:
  - Verifies `outcome="ok"` / `"success"` on valid registration.
  - Verifies `outcome="error"` on duplicate registration attempt.
  - Verifies `outcome="refused"` on ungranted mutation.
- **O4 (State Inspection Parity)**:
  - Compares active layout ID and layout count returned by CLI `aiosh layout list --json` and MCP `aios.fs_layout.list`.
- **O5 (Destructive Mutation Flagging)**:
  - Diffs layout with halved partition sizes against base preset, verifying `destructive: true`.
  - Diffs identical layouts, verifying `destructive: false`.

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

- [x] Targeted test passes.
- [x] No regression in existing smoke suites for touched modules.
