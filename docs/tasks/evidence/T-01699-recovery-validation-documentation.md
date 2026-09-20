# T-01699: Kernel Module Management — Recovery & Validation Documentation

## Metadata
- **Task ID**: `T-01699`
- **Sub-Epic**: Kernel Module Management / Recovery & Validation
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementer**: Documentation Subsystem Agent

---

## 1. Documentation Scope
Updated `docs/kernel_module_management.md` with complete Section 13: "Recovery & Validation Engine (Sub-Epic 10)", covering:
1. **Invariant Matrix (KR1..KR6)**: Formally defines rule partition completeness, autoload partition completeness, health consistency, conflict resolution, non-destructive quarantine, and partial corruption recovery.
2. **Operational CLI Examples**:
   - `aiosh mod check` (read-only health check)
   - `aiosh mod check --store <path> --json` (machine-readable audit output)
   - `aiosh mod check --auto-recover` (self-healing repair with quarantine)
3. **MCP Tool Invocation**:
   - `aios.kernel_module.check` schema and example payloads for both read and mutating recovery calls.
4. **Security Controls**:
   - 10 MB maximum file size cap (`MAX_STORE_FILE_SIZE`).
   - Regular file verification (`is_file`).
   - Path control character sanitization.
   - Audit-ring logging and PEP authorization.
5. **Constraints & Known Limitations**:
   - Distinction between user-space configuration files (`.aios/kernel_modules.json`) and running kernel memory state (`/proc/modules`).
   - Filesystem write permission requirements for quarantine creation and atomic updates.

---

## 2. Acceptance Criteria Checklist
- [x] Documentation updated with copy-pasteable CLI and MCP examples.
- [x] Invariants KR1..KR6 formally documented.
- [x] Security controls and constraints honestly recorded.
- [x] Evidence files linked.
