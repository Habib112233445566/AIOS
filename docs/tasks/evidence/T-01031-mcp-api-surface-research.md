# T-01031 — Distro Selection & Justification / MCP/API Surface: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / MCP/API Surface

## 1. Objectives & Scope
Research the Model Context Protocol (MCP) JSON-RPC 2.0 tool endpoints for AI agents to query, inspect, and evaluate target Linux distributions for AIOS.
- Ensure strict JSON-RPC 2.0 wire conformance.
- Bind all operations through the Policy Enforcement Point (`dispatch::recorded_call`) ensuring Constitution rule verification (R-01..R-12).
- Ensure SHA-256 hash-chained immutable audit recording in `AuditRing` for every agent tool invocation.

## 2. Tool Interfaces & Wire Contracts
```json
{
  "tools": [
    {
      "name": "aios.distro.list",
      "description": "List all registered Linux distribution profiles for AIOS",
      "inputSchema": {
        "type": "object",
        "properties": {
          "store_path": { "type": "string", "description": "Optional custom store path" }
        }
      }
    },
    {
      "name": "aios.distro.show",
      "description": "Get detailed metadata for a specific Linux distribution profile",
      "inputSchema": {
        "type": "object",
        "properties": {
          "id": { "type": "string", "description": "Profile ID" },
          "store_path": { "type": "string", "description": "Optional custom store path" }
        },
        "required": ["id"]
      }
    },
    {
      "name": "aios.distro.evaluate",
      "description": "Run multi-factor evaluation scoring for a distro profile or all profiles",
      "inputSchema": {
        "type": "object",
        "properties": {
          "id": { "type": "string", "description": "Optional profile ID to evaluate" },
          "store_path": { "type": "string", "description": "Optional custom store path" }
        }
      }
    },
    {
      "name": "aios.distro.recommend",
      "description": "Get the recommended production reference Linux distro profile for AIOS",
      "inputSchema": {
        "type": "object",
        "properties": {
          "store_path": { "type": "string", "description": "Optional custom store path" }
        }
      }
    }
  ]
}
```

## 3. Concrete Example Tool Call
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.distro.evaluate",
    "arguments": {
      "id": "debian-12-minimal-x86_64"
    }
  }
}
```

## 4. Failure Modes & Invariants
- Missing required `id` argument in `aios.distro.show` returns JSON-RPC `-32602` (Invalid params).
- Non-existent profile IDs return standard error envelope with `ok: false`.
- Arbitrary store path inputs are bounded by `MAX_STORE_BYTES` (10 MiB) before parsing.
