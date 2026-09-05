# T-01239: Package Management - MCP/API Surface: Documentation

## Metadata
- **Task ID:** `T-01239`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management MCP/API Surface Documentation
- **Status:** Complete

## 1. Documentation Summary
The Autonomous Agent MCP/API tool surface for Linux Package Management (`aiosh-mcp`) has been documented in `docs/README.md` (§8.12).

### Documented Tools:
1. `aios.package.validate`: Validates package name syntax against PM1 or complete `PackageSpec` against PM1..PM5.
2. `aios.package.list`: Lists packages matching optional filters (`format`, `state`, `pattern`, `limit`).
3. `aios.package.get`: Retrieves full `PackageSpec` object by package name.
4. `aios.package.plan`: Computes transactional execution order, verifying dependency closure (CS2, CS3) and size delta calculation (CS4).
5. `aios.package.search`: Substring search across package names and descriptions with pagination limit [1..10,000].
6. `aios.package.apply`: Executes transaction plans with dry-run support, state transitions (CS5), and atomic disk persistence.

## 2. Copy-Pasteable JSON-RPC Examples

### Example 1: `aios.package.search`
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.package.search",
    "arguments": {
      "pattern": "curl",
      "limit": 10
    }
  }
}
```

### Example 2: `aios.package.apply` (Dry Run)
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.package.apply",
    "arguments": {
      "actions": [
        { "action": "install", "package_name": "libssl3" },
        { "action": "install", "package_name": "curl" }
      ],
      "dry_run": true
    }
  }
}
```

## 3. Stated Constraints & Honest Limitations
1. **Network Independence / Offline Registry**: Current tools operate on local store files or in-memory seeds. Direct network package downloads (`apt update`, `apk add` via HTTP) are deferred to later networking/builder milestones.
2. **Dependency Resolution Scope**: Complex transitive multi-level dependency graphs must be provided in the transaction action batch; automatic backtracking SAT-solver resolution is not currently implemented in this phase.
3. **Store Capacity Ceiling**: In-memory store and file reader enforce a 10,000 package entity count ceiling and 10 MiB byte limit.
4. **POSIX Naming Standard**: Package names must conform to `^[a-z0-9][a-z0-9+.-]{1,63}$`.

## 4. Linked Evidence Artifacts
- Research: `docs/tasks/evidence/T-01231-mcp-api-surface-research.md`
- Specification: `docs/tasks/evidence/T-01232-mcp-api-surface-specification.md`
- Scaffold: `docs/tasks/evidence/T-01233-mcp-api-surface-scaffold.md`
- Implementation: `docs/tasks/evidence/T-01234-mcp-api-surface-implementation.md`
- Unit Tests: `docs/tasks/evidence/T-01235-mcp-api-surface-unit-test.md`
- Integration: `docs/tasks/evidence/T-01236-mcp-api-surface-integration.md`
- Security Review: `docs/tasks/evidence/T-01237-mcp-api-surface-security-review.md`
- Hardening: `docs/tasks/evidence/T-01238-mcp-api-surface-hardening.md`
