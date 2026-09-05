# T-01231: Package Management - MCP/API Surface: Research

## Metadata
- **Task ID:** `T-01231`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management MCP/API Surface Research
- **Status:** Complete

## 1. Existing Codebase Survey
An audit of `code/aiosh-rust/aiosh-mcp/src/main.rs` and `code/aiosh-rust/aiosh-core/src/package_service.rs` revealed:
1. **Registered MCP Tools**:
   - `aios.package.validate`: Validates package names or full `PackageSpec` against PM1..PM5.
   - `aios.package.list`: Enumerates packages in store with optional `format`, `state`, `pattern`, `limit` query parameters.
   - `aios.package.get`: Retrieves full `PackageSpec` by package name.
   - `aios.package.plan`: Computes a deterministic transaction plan, builds dependency closure, and calculates size delta.
2. **Identified MCP Tool Surface Gaps**:
   - **Search (`aios.package.search`)**: Currently missing. Operators can use `aiosh package search <pattern>`, but autonomous AI agents on the MCP bus lack a dedicated search tool for substring queries across package names and descriptions.
   - **Apply (`aios.package.apply`)**: Currently missing. Operators can use `aiosh package apply`, but AI agents cannot execute transactions to mutate package store state or commit updates to disk via MCP.

## 2. Authoritative Sources & Prior Art
1. **Model Context Protocol (MCP) Specification (2024-11-05)**:
   - `tools/list`: Declares tool metadata and JSON Schema for inputs (`inputSchema`).
   - `tools/call`: Executes tool with JSON arguments and returns structured content (`content: [{ type: "text", text: ... }]`, `isError`).
2. **PackageKit DBus Interface (freedesktop.org)**:
   - `org.freedesktop.PackageKit.Transaction.SearchNames(flags, values)` / `SearchDetails(flags, values)`.
   - `org.freedesktop.PackageKit.Transaction.InstallPackages(flags, package_ids)` / `RemovePackages(flags, package_ids)`.
   - Clean separation between search, planning, and transactional application.
3. **AIOS Architecture Decisions (ADR-0035 & PEP)**:
   - All tool invocations must route through `dispatch::recorded_call`, binding `actor_id`, `actor`, `grant_id`, and `is_write`.
   - Consequential state mutations (`is_write: true`) write an immutable audit record before and after invocation.

## 3. Fact vs. Assumption Separation

| Item | Status | Details |
| :--- | :--- | :--- |
| Core Store APIs | **Fact** | `PackageStore::search_packages(pattern, limit)` and `PackageStore::apply_transaction(tx)` already exist and are tested in `aiosh-core`. |
| Dispatch & Audit | **Fact** | `dispatch::recorded_call` handles PEP gating and SQLite WAL hash-chained audit logging in `aiosh-mcp`. |
| Read vs. Write Flag | **Fact** | Read-only operations (`validate`, `list`, `get`, `plan`, `search`) pass `is_write: false`. State-mutating operations (`apply`) must pass `is_write: true`. |
| Agent Search Workflow | **Assumption** | Autonomous agents require search to discover package names matching semantic keywords (e.g. `ssl`, `network`, `shell`) without downloading entire store catalogues. |
| Agent Apply Workflow | **Assumption** | Agents should be able to supply either pre-computed transaction plans (`plan`) or raw actions (`actions`), allowing single-step and two-step workflows. |
| Dry Run Default | **Assumption** | When `dry_run` is unspecified in `aios.package.apply`, it should default to `false` unless explicitly requested. |

## 4. Unknowns & Decisions Needed Prior to Implementation

### Decision 1: Input Schema for `aios.package.search`
- **Options**:
  - A: Accept `pattern` (string, required), `limit` (integer, optional, 1..1000), `store_path` (string, optional).
  - B: Accept only `query` without limit.
- **Decision**: Option A. Follows existing `aios.package.list` patterns, allows optional store path, and bounds results.

### Decision 2: Input Schema for `aios.package.apply`
- **Options**:
  - A: Require both `plan` and `actions`.
  - B: Accept either `actions` (array of action objects) or `plan` (complete transaction object), plus optional `dry_run` (boolean, default false), `store_path` (string, optional).
- **Decision**: Option B. Mirrors the CLI surface (`aiosh package apply (--actions ... | --plan ...)`), supporting both direct execution and verified pre-planned execution.

### Decision 3: Atomic Store Persistence on Apply
- **Decision**: If `store_path` is specified and `dry_run == false`, `aios.package.apply` will call `store.save_to_path(&path)` to atomically update the file on disk. If `store_path` is omitted, the transaction executes in-memory.

## 5. Summary
The research phase is complete with no source code modifications made. All decisions and contracts are documented and ready for formal specification in `T-01232`.
