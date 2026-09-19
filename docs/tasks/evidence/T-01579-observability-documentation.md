# T-01579 — Filesystem Layout observability: Documentation

## Metadata
- **Task ID:** `T-01579`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / observability
- **Status:** Complete — documented observability invariants, invocations, constraints, and limitations in `docs/filesystem_layout.md`.
- **Date:** 2026-09-19
- **Depends on:** `T-01578` (Hardening)
- **Feeds:** `T-01580` (Verification & Evidence)
- **Artifacts:** `docs/tasks/evidence/T-01579-observability-documentation.md`, `docs/tasks/evidence/T-01579-documentation.md`

---

## 1. Documentation Updates

Updated `docs/filesystem_layout.md` with:
1. **Sub-Epic 8 Overview & Invariants (FL12)**:
   - O1: Telemetry emission completeness (CLI and MCP).
   - O2: Audit correlation and queryability by target and tool.
   - O3: Outcome fidelity (`ok`/`success`, `error`, `refused`).
   - O4: State inspection parity between CLI and MCP.
   - O5: Destructive mutation flagging (`destructive: true` on partition shrink/delete).
2. **Copy-Pasteable Example Invocations**:
   - `aiosh audit tail --json -n 20`
   - `aiosh layout list --json`
   - Standalone test runner: `python code/aiosh-cli/tests/test_fs_layout_observability.py`
   - Aggregate test battery: `python tools/test_fs_layout_suites.py`
3. **Honest Limitations & Constraints**:
   - Local storage boundary of SQLite WAL `audit.db`.
   - Bounded query limit enforcement (`-n <count>`).
4. **Task Evidence Cross-Links**:
   - Linked all tasks `T-01571` through `T-01580`.

---

## 2. Acceptance Confirmation

- [x] Docs updated with working example.
- [x] Limitations are stated, not omitted.
