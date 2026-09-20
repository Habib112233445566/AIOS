# T-02032: Capability Model / MCP/API Surface — Specification

**Task ID**: `T-02032`  
**Phase**: Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic**: Sub-Epic 4: Capability Model / MCP/API Surface  
**Status**: COMPLETED  
**Date**: 2026-09-20  

---

## 1. Specification Overview

This specification establishes the MCP tool interface contract for the AIOS Capability Model. It adheres strictly to ADR-0035 §D-2 (MCP as external model tool-call interface) and ADR-0035 §A F-2 (PEP gating and Audit Ring recording for all consequential operations).

The surface introduces seven dedicated MCP tools under the `aios.capability.*` namespace:
1. `aios.capability.list`: Enumerate capabilities with optional filtering.
2. `aios.capability.get`: Fetch a capability by ID.
3. `aios.capability.issue`: Issue a root capability (consequential, PEP-gated).
4. `aios.capability.attenuate`: Attenuate a child capability (consequential, PEP-gated).
5. `aios.capability.revoke`: Cascade revoke a capability and its descendants (consequential, PEP-gated).
6. `aios.capability.check`: Fast permission check against registered capabilities.
7. `aios.capability.prune`: Prune expired leaf capabilities (consequential, PEP-gated).

---

## 2. Shared Data Models & Enums

### 2.1 Scope Types (`scope_type`)
- `filesystem`: Path-based resource scope (`scope_target`: file or directory path).
- `network`: Host/port or CIDR scope (`scope_target`: e.g. `127.0.0.1:8080`, `api.anthropic.com:443`).
- `process`: Process or binary scope (`scope_target`: e.g. `/usr/bin/git`, PID string).
- `audit`: Audit subsystem scope (`scope_target`: e.g. `ring`, `tail`, `segments`).
- `pentest`: Pentest tool scope (`scope_target`: e.g. `nmap`, `tshark`).
- `system`: System maintenance scope (`scope_target`: e.g. `update`, `toolchain`).

### 2.2 Capability Rights (`rights`)
- `read`: Read access to the target resource.
- `write`: Modify or append access to the target resource.
- `execute`: Execution rights on the target resource.
- `delegate`: Ability to attenuate and derive child capabilities.
- `admin`: Full administrative control over the target resource.

---

## 3. Tool Specifications

### 3.1 `aios.capability.list`
- **Description**: List registered capabilities with optional subject or active-only filtering.
- **Consequential**: No (read-only).
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "subject": { "type": "string", "description": "Optional filter by subject identifier" },
      "active_only": { "type": "boolean", "description": "Filter to non-revoked, non-expired capabilities (default: false)" },
      "store_path": { "type": "string", "description": "Optional path to capability_store.json (default: .aios/capability_store.json)" }
    },
    "additionalProperties": false
  }
  ```
- **Output Envelope**:
  ```json
  {
    "ok": true,
    "tool": "aios.capability.list",
    "count": 2,
    "capabilities": [ { ... } ]
  }
  ```

### 3.2 `aios.capability.get`
- **Description**: Retrieve detailed capability metadata by capability ID (`CAP-<uuid>`).
- **Consequential**: No (read-only).
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "id": { "type": "string", "description": "Capability identifier (e.g. CAP-12345678-1234-1234-1234-123456789abc)" },
      "store_path": { "type": "string", "description": "Optional path to capability_store.json" }
    },
    "required": ["id"],
    "additionalProperties": false
  }
  ```
- **Output Envelope**:
  ```json
  {
    "ok": true,
    "tool": "aios.capability.get",
    "capability": { ... }
  }
  ```
- **Failure Cases**: Returns `ok: false, error: "Capability <id> not found"` if missing.

### 3.3 `aios.capability.issue`
- **Description**: Issue a new root capability with validated scope, rights, and constraints.
- **Consequential**: Yes (PEP-gated, Audit-logged).
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "issuer": { "type": "string", "description": "Issuer identity (must be 'kernel' or 'admin:*')" },
      "subject": { "type": "string", "description": "Subject identity to receive the capability" },
      "scope_type": { "type": "string", "enum": ["filesystem", "network", "process", "audit", "pentest", "system"] },
      "scope_target": { "type": "string", "description": "Target resource pattern or path" },
      "rights": {
        "type": "array",
        "items": { "type": "string", "enum": ["read", "write", "execute", "delegate", "admin"] },
        "minItems": 1
      },
      "max_invocations": { "type": "integer", "minimum": 1, "description": "Optional invocation quota" },
      "quota_bytes": { "type": "integer", "minimum": 1, "description": "Optional byte quota" },
      "expires_in_secs": { "type": "integer", "minimum": 1, "description": "Optional expiration duration in seconds" },
      "store_path": { "type": "string", "description": "Optional path to capability_store.json" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "required": ["issuer", "subject", "scope_type", "scope_target", "rights"],
    "additionalProperties": false
  }
  ```
- **Output Envelope**:
  ```json
  {
    "ok": true,
    "tool": "aios.capability.issue",
    "capability": { ... }
  }
  ```

### 3.4 `aios.capability.attenuate`
- **Description**: Derive an attenuated child capability from an existing active parent capability.
- **Consequential**: Yes (PEP-gated, Audit-logged).
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "parent_id": { "type": "string", "description": "Parent capability ID" },
      "new_subject": { "type": "string", "description": "Subject receiving attenuated capability" },
      "narrowed_scope_type": { "type": "string", "enum": ["filesystem", "network", "process", "audit", "pentest", "system"] },
      "narrowed_scope_target": { "type": "string", "description": "Narrowed target resource pattern" },
      "subset_rights": {
        "type": "array",
        "items": { "type": "string", "enum": ["read", "write", "execute", "delegate", "admin"] },
        "minItems": 1
      },
      "max_invocations": { "type": "integer", "minimum": 1, "description": "Narrowed invocation quota" },
      "quota_bytes": { "type": "integer", "minimum": 1, "description": "Narrowed byte quota" },
      "expires_in_secs": { "type": "integer", "minimum": 1, "description": "Narrowed expiration duration in seconds" },
      "store_path": { "type": "string", "description": "Optional path to capability_store.json" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "required": ["parent_id", "new_subject", "subset_rights"],
    "additionalProperties": false
  }
  ```
- **Output Envelope**:
  ```json
  {
    "ok": true,
    "tool": "aios.capability.attenuate",
    "capability": { ... }
  }
  ```
- **Failure Cases**: Fails if parent lacks `delegate`, if requested rights exceed parent, if parent revoked or expired.

### 3.5 `aios.capability.revoke`
- **Description**: Revoke a capability and cascade revocation to all derived descendants.
- **Consequential**: Yes (PEP-gated, Audit-logged).
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "id": { "type": "string", "description": "Capability ID to revoke" },
      "store_path": { "type": "string", "description": "Optional path to capability_store.json" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "required": ["id"],
    "additionalProperties": false
  }
  ```
- **Output Envelope**:
  ```json
  {
    "ok": true,
    "tool": "aios.capability.revoke",
    "id": "CAP-root",
    "revoked_ids": ["CAP-root", "CAP-child-1", "CAP-child-2"],
    "count": 3
  }
  ```

### 3.6 `aios.capability.check`
- **Description**: Fast access check verifying if a subject holds an active capability covering the requested scope and right.
- **Consequential**: No by default; marked consequential if `consume: true` is requested.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "subject": { "type": "string", "description": "Subject identity to verify" },
      "scope_type": { "type": "string", "enum": ["filesystem", "network", "process", "audit", "pentest", "system"] },
      "scope_target": { "type": "string", "description": "Target resource" },
      "right": { "type": "string", "enum": ["read", "write", "execute", "delegate", "admin"] },
      "consume": { "type": "boolean", "description": "If true, consume 1 invocation against capability quota" },
      "store_path": { "type": "string", "description": "Optional path to capability_store.json" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID (when consume=true)" }
    },
    "required": ["subject", "scope_type", "scope_target", "right"],
    "additionalProperties": false
  }
  ```
- **Output Envelope**:
  ```json
  {
    "ok": true,
    "tool": "aios.capability.check",
    "granted": true,
    "capability_id": "CAP-12345678-...",
    "remaining_invocations": 9
  }
  ```

### 3.7 `aios.capability.prune`
- **Description**: Prune expired leaf capabilities that have no active children.
- **Consequential**: Yes (PEP-gated, Audit-logged).
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "store_path": { "type": "string", "description": "Optional path to capability_store.json" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "additionalProperties": false
  }
  ```
- **Output Envelope**:
  ```json
  {
    "ok": true,
    "tool": "aios.capability.prune",
    "pruned_count": 5
  }
  ```

---

## 4. Reused vs New Interfaces

- **Reused**:
  - `aiosh_core::capability::{Capability, CapabilityScope, CapabilityRight, CapabilityConstraints}`
  - `aiosh_core::capability_service::CapabilityService`
  - `aiosh_core::dispatch::recorded_call`
  - `aiosh_core::pep::PepStore`
  - `aiosh_core::audit::AuditRing`
- **New**:
  - MCP tool definitions in `code/aiosh-rust/aiosh-mcp/src/main.rs: Server::tool_manifest`
  - Dispatch routing and handlers in `Server::call_tool`
  - Integration smoke tests in `code/aiosh-mcp/tests/test_capability_mcp_smoke.py`
