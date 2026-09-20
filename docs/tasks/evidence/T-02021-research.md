# Task Evidence: T-02021 - Capability Model / CLI surface: Research (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02021`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Research CLI commands, syntax, audit requirements, and security constraints for `aiosh capability`.

---

## 2. CLI Architecture & Integration Research

### 2.1 Existing CLI Patterns in `aiosh-cli`
An examination of `code/aiosh-rust/aiosh-cli/src/main.rs` reveals the established patterns:
1. **Command Routing**: `main()` matches `args.first()` and dispatches to `cmd_<subsystem>(&args[1..])`.
2. **Audit Logging**: Every command invocation records an audit entry to `AuditRing` via `classify_and_emit` or `emit`.
3. **Structured Output**: Commands support both human-readable tabular output and structured JSON (`--json`), conforming to `{ "code": 0, "data": ... }`.
4. **Parameter Parsing**: Uses `has_flag`, `parse_flag`, and positional arguments, with strict input sanitization.
5. **Backing Store**: Defaults to `$AIOSH_HOME/capabilities.json` (or `~/.aios/capabilities.json`), with override via `--store <path>`.

### 2.2 Capability CLI Subcommands Required
| Subcommand | Description | Required Arguments |
|---|---|---|
| `list` | List capabilities in registry, with optional filtering by subject or active status | `[--subject <S>] [--active-only] [--store <PATH>] [--json]` |
| `show` | Display full details of a specific capability | `<ID> [--store <PATH>] [--json]` |
| `issue` | Issue a root capability with verified kernel/admin authorization | `--issuer <I> --subject <S> --scope-type <T> --scope-target <TARGET> [--rights R1,R2] [--expires <ISO8601>] [--max-invocations N] [--quota-bytes B] [--store <PATH>] [--json]` |
| `attenuate` | Derive an attenuated child capability from an existing parent | `--parent <ID> --subject <S> [--scope-type <T>] [--scope-target <TARGET>] [--rights R1,R2] [--store <PATH>] [--json]` |
| `revoke` | Revoke a capability and cascade revocation to all child capabilities | `<ID> [--store <PATH>] [--json]` |
| `check` | Verify whether a subject possesses a valid capability for a scope and right | `--subject <S> --scope-type <T> --scope-target <TARGET> --right <R> [--store <PATH>] [--json]` |
| `prune` | Cleanse expired leaf capabilities with no active child dependencies | `[--store <PATH>] [--json]` |

### 2.3 Security Considerations
1. **Issuer Restraint**: `aiosh capability issue` must strictly require `--issuer kernel` or `--issuer admin:*`. Ambient issuance is refused.
2. **Path Sanitization**: Any custom `--store <PATH>` must be checked with `validate_service_path` before opening.
3. **Error Codes**:
   - `0`: Success
   - `1`: Operational error (not found, unauthorized, invalid parameters)
   - `2`: Syntax / usage / argument validation error
