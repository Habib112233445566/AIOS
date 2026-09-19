# T-01582 — Filesystem Layout documentation: Specification

## Metadata
- **Task ID:** `T-01582`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / documentation
- **Status:** Complete — specified documentation verification contract and test criteria FL13 (D1..D5).
- **Date:** 2026-09-19
- **Depends on:** `T-01581` (Research)
- **Feeds:** `T-01583` (Scaffold)
- **Artifacts:** `docs/tasks/evidence/T-01582-documentation-specification.md`, `docs/tasks/evidence/T-01582-spec.md`

---

## 1. Documentation Contract Specification

### 1.1 Document Structure & Authority
`docs/filesystem_layout.md` is designated the definitive documentation authority for the Filesystem Layout subsystem:
- §1: Architecture, FHS 3.0 alignment, and Kali/Debian integration.
- §2: Canonical presets (`aios-uefi-standard-v1`, `aios-container-minimal-v1`).
- §3: Invariants FL1..FL6 (bootable partition, root mount, swap, tmp/dev/shm security flags, relative symlinks).
- §4: Operator CLI reference for all 11 subcommands with verified walkthrough.
- §5: Agent MCP tool reference for all 10 tools (`aios.fs_layout.*`).
- §6: Exit codes, result envelopes, and 11 honest limitations.
- §7: Sub-Epic history and task evidence cross-links (Sub-Epics 1..9).

---

### 1.2 Automated Documentation Criteria (FL13: D1..D5)

The automated documentation test suite `code/aiosh-cli/tests/test_fs_layout_documentation.py` verifies:
- **D1 (CLI Subcommand Completeness)**: `aiosh layout --help` advertises all subcommands: `list`, `show`, `validate`, `probe`, `diff`, `fstab`, `register`, `set-active`, `remove`, `import-fstab`.
- **D2 (MCP Manifest Schema Parity)**: All 10 tools advertised by `tools/list` have matching parameter lists in `inputSchema.properties` with `additionalProperties: false`.
- **D3 (Evidence Link Integrity)**: All markdown links to `docs/tasks/evidence/T-*.md` in `docs/filesystem_layout.md` resolve to existing files on disk.
- **D4 (JSON Snippet Syntactic Validity)**: All JSON code blocks in `docs/filesystem_layout.md` parse as valid JSON.
- **D5 (Error Code Documentation Completeness)**: Every error code used across CLI and MCP is documented in §4.12.

---

## 2. Reused vs New Interfaces

- **Reused**:
  - `aiosh layout --help` and `aiosh-mcp tools/list`.
  - `docs/filesystem_layout.md` and evidence files in `docs/tasks/evidence/`.
- **New**:
  - `code/aiosh-cli/tests/test_fs_layout_documentation.py`: Test suite for D1..D5.
  - Criterion `FL13` registered in `tools/test_fs_layout_suites.py`.

---

## 3. Acceptance Confirmation

- [x] Spec covers happy path, failure path, and audit effects.
- [x] Spec is reviewable without reading the implementation.
