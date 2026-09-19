# T-01587 — Filesystem Layout documentation: Security Review

## Metadata
- **Task ID:** `T-01587`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / documentation
- **Status:** Complete — conducted security review of filesystem layout documentation assets.
- **Date:** 2026-09-19
- **Depends on:** `T-01586` (Documentation Integration)
- **Feeds:** `T-01588` (Documentation Hardening)
- **Artifacts:** `docs/tasks/evidence/T-01587-documentation-security-review.md`, `docs/tasks/evidence/T-01587-security.md`

---

## 1. Security Review Scope & Objectives

Review the security properties and integrity of the Filesystem Layout documentation:
1. Markdown injection, stored XSS, and untrusted markup handling.
2. Link traversal and path security in documentation references.
3. Safe copy-paste command examples (prevention of command injection traps).
4. Schema synchronization and prevention of misleading/desynchronized instructions.

---

## 2. Abuse Scenarios Analyzed

### Scenario D-A1: Markdown / HTML Injection
- **Vector**: Injection of raw HTML, `<script>`, or `javascript:` links into documentation files.
- **Defense**: All documentation is standard GitHub Flavored Markdown (GFM). Rendering targets sanitize raw HTML and prohibit executable script execution.
- **Verdict**: Mitigated — reviewed and confirmed clean.

### Scenario D-A2: Link Traversal & Host Disclosure
- **Vector**: Documentation links attempt path traversal (`../../`) to expose host files outside the repository.
- **Defense**: All task links in `docs/filesystem_layout.md` point strictly to in-tree files within `docs/tasks/evidence/` (verified by FL13 D3).
- **Verdict**: Mitigated — tested by FL13 D3.

### Scenario D-A3: Copy-Paste Traps & Command Injection
- **Vector**: Example shell commands contain hidden control characters, malicious expansions, or destructive targets.
- **Defense**: All documented commands use safe, isolated AIOS subcommands (`aiosh layout ...`, `aiosh grant ...`) with explicit placeholder tokens (`<GRANT_ID>`).
- **Verdict**: Mitigated — verified by review.

### Scenario D-A4: Schema Desynchronization
- **Vector**: Inaccurate or missing argument documentation leads agents to submit invalid or unvalidated parameters.
- **Defense**: Automated test suite `test_fs_layout_documentation.py` (FL13) mechanically asserts that CLI `--help` and MCP `tools/list` match documented interfaces.
- **Verdict**: Mitigated — tested by FL13 D1, D2, and D5.

---

## 3. Acceptance Confirmation

- [x] Security evidence file exists with abuse scenarios.
- [x] No known policy bypass remains open.
