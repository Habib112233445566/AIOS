# T-01729: Hardware Detection — CLI Surface Documentation

## Metadata
- **Task ID**: `T-01729`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Documentation Updates
Section 9 ("Hardware Detection CLI Surface (Sub-Epic 3)") was added to `docs/hardware_detection.md` detailing:
1. **Command Syntax**: Detailed usage of `aiosh hw <subcommand>` and alias `aiosh hardware`.
2. **Subcommands**: Comprehensive parameter, flag, and output description for `scan`, `list`, `show`, `summary`, and `verify`.
3. **Flags & Options**: Options `--class`, `--no-attrs`, `--sysfs`, `--procfs`, `--file`, and `--json`.
4. **Exit Codes**: Standardized mapping: `0` (Success), `1` (Domain failure), and `2` (CLI invocation / parameter error).
5. **Security & Audit Invariants**: Documentation of invariants CS1..CS4 (terminal sanitization, device ID hygiene, buffer limits, and SQLite WAL audit logging).
6. **Traceability**: Complete link matrix to Sub-Epic 3 evidence artifacts `T-01721` through `T-01730`.
