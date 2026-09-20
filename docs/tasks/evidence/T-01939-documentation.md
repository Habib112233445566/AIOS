# Task Evidence: T-01939 - System Update / MCP/API surface: Documentation

- **Task**: `T-01939`
- **Sub-Epic**: `Sub-Epic 4: Model Context Protocol (MCP) & API Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Documentation
Authored comprehensive documentation for the System Update Model Context Protocol (MCP) & Agent API Subsystem in `docs/system_update.md` (Section 7):
- **Tool Manifest & Schemas**: Documented all 6 tools (`aios.update.status`, `aios.update.slots`, `aios.update.check`, `aios.update.apply`, `aios.update.confirm`, `aios.update.rollback`) with input parameters and structured response payloads.
- **Copy-Pasteable JSON-RPC Examples**: Provided working examples for querying status, checking inline manifests, and confirming boots.
- **Operational Invariants**: Explicitly documented invariants `UMCP1` through `UMCP6` covering schema validation, path hygiene, memory/symlink bounds, PEP policy gating, audit trail emission, and state machine lifecycle gating.
- **Constraints & Limitations**: Honestly stated requirements for state directory overrides in test environments, deferral of physical reboot to system reboot controllers, and 100% staging artifact digest match requirements.
- **Task Evidence Links**: Linked evidence files `T-01931` through `T-01938`.

## Verification
- Verified section structure, markdown links, code blocks, and table formatting in `docs/system_update.md`.
