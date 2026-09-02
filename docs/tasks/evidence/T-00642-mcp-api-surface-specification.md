# T-00642 — Repository Health / MCP/API surface: Specification

## 1. Specification Overview
The `aios.repo.health` MCP tool provides AI agents and external models with a standardized JSON-RPC interface to query repository health diagnostics across all supported check domains.

## 2. Tool Definition & Schema

### 2.1 Manifest Declaration (`tools/list`)
```json
{
  "name": "aios.repo.health",
  "description": "Assess repository health, Git working tree cleanliness, file bounds, and security governance policies.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "repo_path": {
        "type": "string",
        "description": "Target repository root directory (default: current directory .)."
      }
    },
    "additionalProperties": false
  }
}
```

### 2.2 Tool Call Invocation (`tools/call`)
**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "method": "tools/call",
  "params": {
    "name": "aios.repo.health",
    "arguments": {
      "repo_path": "."
    }
  }
}
```

**Success Response Envelope:**
```json
{
  "ok": true,
  "tool": "aios.repo.health",
  "report": {
    "repo_path": ".",
    "timestamp_utc": "2026-08-29T12:00:00Z",
    "overall_status": "Pass",
    "total_checks": 3,
    "passed_checks": 3,
    "warn_checks": 0,
    "failed_checks": 0,
    "skipped_checks": 0,
    "checks": [ ... ]
  }
}
```

**Failure Response Envelope:**
```json
{
  "ok": false,
  "tool": "aios.repo.health",
  "error": "Failed to assess repository health: <reason>"
}
```

## 3. PEP Policy & Audit Invariants
- **PEP Enforcement**: Read-only diagnostic tool. No grant token required.
- **Audit Emission**: Automatically logs an audit record with `tool: "aios.repo.health"`, `actor: "agent"`, and `c_flags` clear.
