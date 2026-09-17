# T-01538 — Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / MCP/API surface: Hardening

Completed: 2026-09-17T21:45:22Z

Acceptance criteria:
- [x] Failure modes produce explicit, auditable errors.
- [x] No temp/connection leaks on the error path.

Scope of those ticks: both criteria hold **for the fs_layout surface this task owns**, and are
established by the probes in `T-01538-mcp-api-surface-hardening.md`. They do **not** extend to the two
residuals that file names as deliberately open — read-tool gating (`T-01537` F-4) and the
classifier/CLI-provenance read side (`T-01537` F-2) — nor to the audit ring's own durability defects
(F-02/F-06/F-16 of `docs/SECURITY-AUDIT-2026-09-18.md`), which live in `dispatch`/`audit` and are
shared by ~130 tools. See that evidence file's §5 before reading this task as closing them.
