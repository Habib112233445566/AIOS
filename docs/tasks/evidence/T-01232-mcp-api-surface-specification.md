# T-01232: Package Management - MCP/API Surface: Specification

## Metadata
- **Task ID:** `T-01232`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management MCP/API Surface Specification
- **Status:** Complete

## 1. Scope & Existing Interface Reuse
This specification standardizes the AIOS Model Context Protocol (MCP) tool surface for Package Management:
- **Reused Interfaces**:
  - `aios.package.validate`: Validates package names or full `PackageSpec` against PM1..PM5.
  - `aios.package.list`: Enumerates packages in store with optional format, state, pattern, limit filters.
  - `aios.package.get`: Retrieves full `PackageSpec` by package name.
  - `aios.package.plan`: Plans transactions with dependency closure validation and delta calculation.
- **New Interfaces (AIOS-Specific Extensions)**:
  - `aios.package.search`: Substring querying on package names and descriptions.
  - `aios.package.apply`: Transaction application, state mutation, and optional disk persistence.

---

## 2. Interface Specifications

### 2.1 Tool: `aios.package.search`
Search package store by substring on package name or description.

#### Input Schema (`inputSchema`)
```json
{
  "type": "object",
  "properties": {
    "pattern": {
      "type": "string",
      "description": "Substring search pattern for package name or description"
    },
    "limit": {
      "type": "integer",
      "description": "Optional maximum number of packages to return (default 50, max 1000)"
    },
    "store_path": {
      "type": "string",
      "description": "Optional path to custom package store JSON file"
    },
    "grant_id": {
      "type": "string",
      "description": "Optional PEP authorization grant ID"
    }
  },
  "required": ["pattern"],
  "additionalProperties": false
}
```

#### Happy Path Output
```json
{
  "ok": true,
  "tool": "aios.package.search",
  "pattern": "curl",
  "matches": 1,
  "packages": [
    {
      "name": "curl",
      "version": "7.88.1-10+deb12u5",
      "architecture": "amd64",
      "format": "deb",
      "state": "available",
      "description": "command line tool for transferring data with URL syntax",
      "installed_size_bytes": 4194304,
      "sha256": "2222222222222222222222222222222222222222222222222222222222222222",
      "repository_url": "https://deb.debian.org/debian",
      "dependencies": [
        { "name": "libc6", "version_constraint": ">= 2.36", "optional": false },
        { "name": "libssl3", "version_constraint": ">= 3.0.0", "optional": false }
      ]
    }
  ]
}
```

#### Error Handling & Audit Effects
- Missing `pattern`: Returns `Err("Missing 'pattern' parameter")`.
- Pattern length > 256 or control characters: Returns `Err("Invalid search pattern")`.
- Audit: Recorded with `is_write: false` in `audit.db` via `dispatch::recorded_call`.

---

### 2.2 Tool: `aios.package.apply`
Applies a package transaction to the store, executing state transitions and optionally persisting updates to disk.

#### Input Schema (`inputSchema`)
```json
{
  "type": "object",
  "properties": {
    "actions": {
      "type": "array",
      "items": { "type": "object" },
      "description": "List of package actions to execute in the transaction (mutually exclusive with 'plan')"
    },
    "plan": {
      "type": "object",
      "description": "Pre-computed PackageTransaction object to apply (mutually exclusive with 'actions')"
    },
    "dry_run": {
      "type": "boolean",
      "description": "Whether to execute as a dry-run without mutating store or disk state (default false)"
    },
    "store_path": {
      "type": "string",
      "description": "Optional path to package store JSON file to update and persist"
    },
    "grant_id": {
      "type": "string",
      "description": "Optional PEP authorization grant ID"
    }
  },
  "additionalProperties": false
}
```

#### Happy Path Output
```json
{
  "ok": true,
  "tool": "aios.package.apply",
  "dry_run": false,
  "persisted": true,
  "report": {
    "transaction_id": "9f83...e12a",
    "packages_installed": ["libssl3", "curl"],
    "packages_removed": [],
    "packages_upgraded": [],
    "total_size_delta_bytes": 9437184,
    "success": true,
    "error": null,
    "timestamp": "2026-09-04T19:22:00Z"
  }
}
```

#### Error Cases & Invariant Protections
- **No Input**: Neither `actions` nor `plan` provided -> Returns `Err("Either 'actions' or 'plan' parameter must be provided")`.
- **Dependency Closure Violation**: Missing prerequisites -> Returns `Err("invariant CS3 violated: ...")`.
- **Tampering / Arithmetic Mismatch**: Plan delta does not match recomputed state delta -> Returns `Err("invariant CS4 violated: ...")`.
- **Disk Persistence Failure**: Write failure on `store_path` -> Returns `Err("failed to persist store: ...")`.

#### Audit & PEP Effects
- Gated as a consequential write operation (`is_write: true`).
- Records structured start and completion events in SQLite WAL `audit.db` with actor identity and payload metadata.

---

## 3. Summary
The specification establishes complete parity between the operator CLI and autonomous agent MCP surfaces while preserving all security, audit, and transactional invariants.
