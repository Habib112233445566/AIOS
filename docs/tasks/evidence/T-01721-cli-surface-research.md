# T-01721: Hardware Detection — CLI Surface Research

## Metadata
- **Task ID**: `T-01721`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Operator Surface Research & CLI Design
Researched CLI operator surface patterns and requirements in `code/aiosh-rust/aiosh-cli`:

1. **Subcommand Namespace**:
   - `aiosh hw` and `aiosh hardware` alias:
     - `aiosh hw scan [--class <class>] [--no-attrs] [--sysfs <path>] [--procfs <path>] [--json]`
     - `aiosh hw list [--class <class>] [--sysfs <path>] [--procfs <path>] [--json]`
     - `aiosh hw show <device-id> [--sysfs <path>] [--procfs <path>] [--json]`
     - `aiosh hw summary [--sysfs <path>] [--procfs <path>] [--json]`
     - `aiosh hw verify [--file <path>] [--sysfs <path>] [--procfs <path>] [--json]`

2. **JSON Envelope & Exit Code Discipline**:
   - Every `--json` response emits a canonical `{ "code": <int>, "data": <val>, "error": <err> }` envelope.
   - Exit code 0: Command succeeded.
   - Exit code 1: Domain operation failure (e.g., `DEVICE_NOT_FOUND`, `INVALID_INVENTORY_INVARIANTS`).
   - Exit code 2: CLI usage / argument syntax / path boundary failure (e.g., `UNKNOWN_SUBCOMMAND`, `MISSING_DEVICE_ID`, `PATH_TOO_LONG`, `PATH_CONTAINS_CONTROL_CHAR`).

3. **Audit Ring & PEP Integration**:
   - Every CLI execution records an audit row using `emit(&mut ctx, "hardware", command, args, outcome, target, detail, actor, ...)` in the local SQLite WAL ring.

4. **Cross-Platform Hermetic Testing Support**:
   - Supports `--sysfs <path>` and `--procfs <path>` flags enabling CLI smoke tests and integration pipelines on non-Linux hosts.

---

## 2. Acceptance Criteria Checklist
- [x] Researched subcommands (`scan`, `list`, `show`, `summary`, `verify`).
- [x] Standard envelope and exit codes mapped.
- [x] Audit ring integration identified.
- [x] Hermetic testing paths identified.
