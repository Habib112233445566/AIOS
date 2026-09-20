# Task Evidence: T-01931 - System Update / MCP/API surface: Research

- **Task**: `T-01931`
- **Sub-Epic**: `Sub-Epic 4: Model Context Protocol (MCP) & API Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Research Findings & Architectural Decisions

### 1. Protocol Architecture & Tool Surface
The AIOS Model Context Protocol (MCP) server exposes a stdio-based JSON-RPC interface for external agents and orchestrators.
Six distinct tools are established for the System Update subsystem:
1. `aios.update.status`: Read-only tool querying update state machine, versions, and progress.
2. `aios.update.slots`: Read-only tool inspecting physical A/B slot health, active slot, and rollback slot.
3. `aios.update.check`: Verification tool accepting either a manifest file path or inline manifest JSON.
4. `aios.update.apply`: Consequential tool triggering payload verification and setting candidate boot slot.
5. `aios.update.confirm`: Consequential tool validating running version and confirming boot success on active slot.
6. `aios.update.rollback`: Consequential tool reverting boot slot to fallback partition.

### 2. Operational Invariants (UMCP1..UMCP6)
- **`UMCP1` (JSON-RPC Schema Compliance)**: All input schemas define strict types, bounds, and `additionalProperties: false`.
- **`UMCP2` (PEP Policy & Grant Gating)**: Consequential operations (`apply`, `confirm`, `rollback`) require Policy Enforcement Point (PEP) evaluation and optional `grant_id`.
- **`UMCP3` (Path & Argument Hygiene)**: Input strings (paths, versions) are sanitized against control characters and capped at safe maximum lengths ($\le 1024$ bytes).
- **`UMCP4` (Audit Ring Interception)**: All tool invocations are logged into the SQLite WAL audit ring via `record_tool_audit` with full parameter capture.
- **`UMCP5` (Cross-Surface State Parity)**: MCP tools operate directly against `SystemUpdateService` state, ensuring 100% data consistency with the operator CLI.
- **`UMCP6` (Deterministic Error Responses)**: All operational errors return structured JSON error payloads without process abort or unhandled exceptions.

## Next Steps
Formulate formal JSON schemas and tool specifications for Sub-Epic 4 Specification (`T-01932`).
