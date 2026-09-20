# Task Evidence: T-02029 - Capability Model / CLI surface: Documentation (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02029`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Document the `aiosh capability` CLI commands, flags, examples, and exit codes in `docs/capability_model.md`.

---

## 2. Documentation Updates
- Updated `docs/capability_model.md`:
  - Appended Section 8: "CLI Surface Reference (`aiosh capability` / `aiosh cap`)".
  - Detailed subcommands: `list`, `show`, `issue`, `attenuate`, `revoke`, `check`, `prune`.
  - Documented common flags: `--store`, `--json`, `-h, --help`.
  - Documented exit codes (0 = success, 1 = operational error, 2 = argument error).
  - Updated Section 6 Evidence Artifacts with links to all Sub-Epic 3 evidence files.
