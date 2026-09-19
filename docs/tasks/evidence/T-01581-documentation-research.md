# T-01581 — Filesystem Layout documentation: Research

## Metadata
- **Task ID:** `T-01581`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / documentation
- **Status:** Complete — researched existing documentation assets, authoritative references, and documentation integrity verification mechanisms.
- **Date:** 2026-09-19
- **Depends on:** `T-01580` (Sub-Epic 8 Milestone Closure)
- **Feeds:** `T-01582` (Documentation Specification)
- **Artifacts:** `docs/tasks/evidence/T-01581-documentation-research.md`, `docs/tasks/evidence/T-01581-research.md`

---

## 1. Existing Documentation Assets

1. **Architecture & Subsystem Reference**:
   - `docs/filesystem_layout.md`: Comprehensive reference detailing data structures, invariants (FL1..FL6), CLI verbs, MCP tool manifests, error codes, and Sub-Epic sections (Sub-Epics 1 through 8).
2. **Repository Index**:
   - `docs/README.md`: Entry §8.14 indexing the Filesystem Layout subsystem.
3. **In-Code Documentation**:
   - Rust doc comments on `aiosh-core::fs_layout`, `aiosh-core::fs_layout_service`, `aiosh-cli::cmd_fs_layout`, and `aiosh-mcp`.
   - CLI `--help` text across `aiosh layout <subcommand>`.
   - MCP `tools/list` JSON-RPC manifest describing all 10 `aios.fs_layout.*` tools.

---

## 2. Authoritative References & Standards

- **Filesystem Hierarchy Standard (FHS 3.0)**: Directory hierarchy and purpose conventions (`/var`, `/etc`, `/usr`).
- **`fstab(5)` & `systemd.mount(5)`**: Standard Linux mount options and format rules.
- **Model Context Protocol (MCP) Specification**: Schema representation (`inputSchema`, `properties`, `required`, `additionalProperties: false`).

---

## 3. Facts vs. Assumptions

| Item | Status | Details |
|---|---|---|
| In-tree docs exist | **Fact** | `docs/filesystem_layout.md` contains 800+ lines covering subcommands, tools, and limitations. |
| Help text & manifest parity | **Fact** | CLI `--help` and MCP `tools/list` expose argument names and subcommands. |
| Automated documentation checks | **Assumption** | An automated suite `test_fs_layout_documentation.py` (Criterion `FL13`) can mechanically assert that doc examples, `--help` output, and manifest properties match actual implementations. |

---

## 4. Decisions Needed

- **Decision 1**: Define Criterion `FL13` to test documentation consistency:
  - D1: CLI `--help` output contains all subcommands.
  - D2: MCP `tools/list` schema matches documented arguments.
  - D3: All referenced evidence files exist on disk.
  - D4: Code examples in `docs/filesystem_layout.md` are syntactically valid.

---

## 5. Acceptance Confirmation

- [x] Evidence file exists and separates facts from assumptions.
- [x] No code changed; decisions needed are listed explicitly.
