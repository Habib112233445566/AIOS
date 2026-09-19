# T-01589 — Filesystem Layout documentation: Documentation

## Metadata
- **Task ID:** `T-01589`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / documentation
- **Status:** Complete — documented documentation test criteria, copy-pasteable examples, and limitations in `docs/filesystem_layout.md`.
- **Date:** 2026-09-19
- **Depends on:** `T-01588` (Hardening)
- **Feeds:** `T-01590` (Verification & Evidence)
- **Artifacts:** `docs/tasks/evidence/T-01589-documentation-documentation.md`, `docs/tasks/evidence/T-01589-documentation.md`

---

## 1. Documentation Updates

Updated `docs/filesystem_layout.md` with:
1. **Sub-Epic 9 Overview & Invariants (FL13)**:
   - D1: CLI subcommand completeness in `aiosh layout --help`.
   - D2: MCP manifest schema parity in `aiosh-mcp tools/list`.
   - D3: Evidence link integrity (all referenced task files exist on disk).
   - D4: JSON code block syntactic validity.
   - D5: Error code documentation completeness (§4.12).
2. **Copy-Pasteable Example Invocations**:
   - `python code/aiosh-cli/tests/test_fs_layout_documentation.py`
   - `python tools/test_fs_layout_suites.py`
3. **Honest Limitations & Constraints**:
   - Distinction between static documentation verification and dynamic runtime behavior.
4. **Task Evidence Cross-Links**:
   - Linked all tasks `T-01581` through `T-01590`.

---

## 2. Acceptance Confirmation

- [x] Docs updated with working example.
- [x] Limitations are stated, not omitted.
