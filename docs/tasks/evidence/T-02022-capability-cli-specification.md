# Task Evidence: T-02022 - Capability Model / CLI surface: Specification (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02022`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Formulate rigorous formal specification for `aiosh capability` CLI commands, flags, outputs, and exit codes.

---

## 2. Command Specification

### 2.1 Routing & Alias
- Root command: `aiosh capability`
- Alias: `aiosh cap`

### 2.2 Subcommands & Arguments

| Subcommand | Positional Args | Key Flags | Description |
|---|---|---|---|
| `list` | None | `--subject <S>`, `--active-only`, `--store <P>`, `--json` | Lists capabilities in registry matching filters. |
| `show` | `<ID>` | `--store <P>`, `--json` | Shows detailed attributes of a capability. |
| `issue` | None | `--issuer <I>`, `--subject <S>`, `--scope-type <T>`, `--scope-target <TG>`, `--recursive`, `--rights <R,...>`, `--expires <ISO>`, `--max-invocations <N>`, `--quota-bytes <B>`, `--store <P>`, `--json` | Issues a new root capability. Issuer must be `kernel` or `admin:*`. |
| `attenuate` | None | `--parent <ID>`, `--subject <S>`, `--scope-type <T>`, `--scope-target <TG>`, `--recursive`, `--rights <R,...>`, `--expires <ISO>`, `--max-invocations <N>`, `--quota-bytes <B>`, `--store <P>`, `--json` | Derives an attenuated child capability. |
| `revoke` | `<ID>` | `--store <P>`, `--json` | Revokes capability and cascades to all child capabilities. |
| `check` | None | `--subject <S>`, `--scope-type <T>`, `--scope-target <TG>`, `--right <R>`, `--store <P>`, `--json` | Verifies access permission for a subject. |
| `prune` | None | `--store <P>`, `--json` | Removes expired leaf capabilities. |

### 2.3 Exit Codes
- `0`: Success
- `1`: Operational error (not found, unauthorized, quota exceeded, revoked)
- `2`: Syntax or validation error (missing required flag, invalid path, bad format)

### 2.4 Audit Invariants
Every subcommand invocation emits a single row to `AuditRing` with:
- `tool`: `"capability"`
- `command`: Subcommand name (`list`, `show`, `issue`, `attenuate`, `revoke`, `check`, `prune`)
- `outcome`: `"success"` or `"failure"`
- `actor`: `"operator"`
