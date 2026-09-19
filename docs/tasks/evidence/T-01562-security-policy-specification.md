# T-01562 — Filesystem Layout security policy: Specification

## Metadata
- **Task ID:** `T-01562`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / security policy
- **Status:** Complete — specification established for the security policy governing Filesystem Layout operations across PEP gating, path confinement, and audit logging.
- **Date:** 2026-09-19
- **Depends on:** `T-01561` (Security Policy Research)
- **Feeds:** `T-01563` (Security Policy Scaffold)
- **Artifacts:** `docs/tasks/evidence/T-01562-security-policy-specification.md`, `docs/tasks/evidence/T-01562-spec.md`

---

## 1. Security Policy Specification

### 1.1 Tool Authorization & PEP Gating Matrix

| Tool / Subcommand | Surface | Gating Policy | Required Scope | Audit Outcome |
|---|---|---|---|---|
| `aios.fs_layout.get` | MCP | Ungated | None | `success` / `error` |
| `aios.fs_layout.list` | MCP | Ungated | None | `success` / `error` |
| `aios.fs_layout.validate` | MCP | Ungated | None | `success` / `error` |
| `aios.fs_layout.probe` | MCP | Ungated | None | `success` / `error` |
| `aios.fs_layout.diff` | MCP | Ungated | None | `success` / `error` |
| `aios.fs_layout.fstab` | MCP | Ungated | None | `success` / `error` |
| `aios.fs_layout.register` | MCP | **PEP Gated** | `scope.tools` contains `aios.fs_layout.*` or `aios.fs_layout.register`; `scope.paths` covers `store_path` and `spec` (if file path) | `success` / `error` / `refused` |
| `aios.fs_layout.set_active` | MCP | **PEP Gated** | `scope.tools` contains `aios.fs_layout.*` or `aios.fs_layout.set_active`; `scope.paths` covers `store_path` | `success` / `error` / `refused` |
| `aios.fs_layout.remove` | MCP | **PEP Gated** | `scope.tools` contains `aios.fs_layout.*` or `aios.fs_layout.remove`; `scope.paths` covers `store_path` | `success` / `error` / `refused` |
| `aios.fs_layout.import_fstab` | MCP | **PEP Gated** | `scope.tools` contains `aios.fs_layout.*` or `aios.fs_layout.import_fstab`; `scope.paths` covers `store_path` and `fstab` file | `success` / `error` / `refused` |
| `aiosh layout <all>` | CLI | Operator Session | None (runs under operator context) | `success` / `error` |

---

### 1.2 Path Confinement & Subject Rules (ADR-0034)

1. **Path Normalization**: All input paths (`store_path`, `spec`, `fstab`) are canonicalized before PEP evaluation.
2. **Allow-List Checking**: If `grant.scope.paths` defines an allow-list, the target path must reside within an allowed directory prefix.
3. **Deny-List Checking**: If target path matches any entry in `grant.scope.paths.deny`, the call is immediately refused with `outcome="refused"`.
4. **Canonical Evasion Resistance**: Path matching resolves 8.3 short aliases, case variations, trailing separators, and symbolic links before checking.

---

### 1.3 Audit Logging Contract (ADR-0035)

1. Every tool invocation writes exactly one SHA-256 hash-chained record to SQLite WAL `audit.db`.
2. On PEP refusal:
   - Tool execution is aborted before touching storage.
   - Audit row records `outcome="refused"`, `outcome_detail` stating the exact scope violation.
3. Hash chain integrity: Each row links `prev_hash` to the preceding row's `hash`.

---

## 2. Acceptance Confirmation

- [x] Spec covers happy path, failure path, and audit effects.
- [x] Spec is reviewable without reading the implementation.
