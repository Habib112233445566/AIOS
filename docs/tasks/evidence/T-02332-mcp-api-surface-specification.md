# T-02332: Audit Chain Extensions MCP/API Surface Specification

## Overview
This specification details the tool contracts, input schemas, response structures, and audit invariants for the Audit Chain Extensions tools exposed via the Model Context Protocol in `aiosh-mcp`.

## MCP Tool Specifications

### 1. `aios.audit.query`
- **Description**: Query extended audit events by provenance, session, trace, actor, tool, or causal parent.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "session_id": { "type": "string" },
      "trace_id": { "type": "string" },
      "actor": { "type": "string" },
      "tool": { "type": "string" },
      "parent_hash": { "type": "string" },
      "limit": { "type": "integer" }
    }
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "tool": "aios.audit.query",
    "count": 1,
    "rows": [ ... ]
  }
  ```

### 2. `aios.audit.inspect`
- **Description**: Fetch detailed record and metadata for a specific audit row by its SHA-256 hash.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "hash": { "type": "string" }
    },
    "required": ["hash"]
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "tool": "aios.audit.inspect",
    "row": { ... }
  }
  ```

### 3. `aios.audit.ancestry`
- **Description**: Trace causal DAG lineage upwards to root triggers.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "hash": { "type": "string" },
      "depth": { "type": "integer" }
    },
    "required": ["hash"]
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "tool": "aios.audit.ancestry",
    "target_hash": "...",
    "ancestors": [ ... ],
    "max_depth_reached": false
  }
  ```

### 4. `aios.audit.sign_verify`
- **Description**: Verify digital signature attached to an audit event.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "hash": { "type": "string" }
    },
    "required": ["hash"]
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "tool": "aios.audit.sign_verify",
    "target_hash": "...",
    "has_signature": true,
    "algorithm": "ed25519",
    "is_valid": true,
    "error": null
  }
  ```

## Audit Side Effects
All tool invocations are recorded in the continuous SQLite WAL audit ring via `dispatch::recorded_call`.
