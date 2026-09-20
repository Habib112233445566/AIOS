# Task Evidence: T-02024 - Capability Model / CLI surface: Implementation (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02024`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Implement all `aiosh capability` CLI subcommands (`list`, `show`, `issue`, `attenuate`, `revoke`, `check`, `prune`) in `code/aiosh-rust/aiosh-cli/src/main.rs`.

---

## 2. Implementation Details

### 2.1 Scope & Rights Parsing
- `parse_cli_scope(scope_type: &str, target: &str, recursive: bool)`:
  - Maps `filesystem` / `fs`, `network` / `net`, `tool`, `process` / `proc`, `ipc`, `system` / `sys` into `CapabilityScope`.
- `parse_cli_rights(rights_str: &str)`:
  - Parses comma-separated rights (`read`, `write`, `execute` / `exec`, `delete` / `del`, `admin`, `delegate`) into `Vec<CapabilityRight>`.

### 2.2 Subcommands Implemented
1. `list`: Lists capabilities with optional `--subject` and `--active-only` filters.
2. `show <id>`: Displays full metadata and constraint consumption for a specific capability.
3. `issue`: Issues root capability with `--issuer`, `--subject`, `--scope-type`, `--scope-target`, and rights. Enforces issuer authorization (`kernel` or `admin:*`).
4. `attenuate`: Attenuates child capability with `--parent`, `--subject`, `--rights`, and optional narrowed scope/constraints.
5. `revoke <id>`: Revokes capability and transitively revokes all descendant capabilities in the registry.
6. `check`: Verifies subject authorization for a specified scope and right, returning exit code 0 if granted, 1 if denied.
7. `prune`: Cleanses expired leaf capabilities with no active child references.

### 2.3 Output & Audit Conformance
- Supports both human-readable text and `--json` structured output envelopes `{ "code": 0/1/2, "data": ..., "error": ... }`.
- Emits audit events to `AuditRing` with tool `"capability"` for every command invocation.
