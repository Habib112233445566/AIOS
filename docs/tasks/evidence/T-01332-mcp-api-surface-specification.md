# T-01332: Init & Service Supervision - MCP/API Surface: Specification

## Metadata
- **Task ID:** `T-01332`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface
- **Status:** Complete (Specification)

---

## 1. Scope & Overview
This specification defines the exact contract, data types, JSON Schemas, invocation semantics, failure handling, persistence behavior, and security audit guarantees for the Model Context Protocol (MCP) tool surface governing Init & Service Supervision (`aios.service.*`).

All tools adhere to JSON-RPC 2.0 transport over standard I/O (stdio) as defined by the MCP specification, with policy enforcement (PEP) and audit emission mandated by ADR-0035 and ADR-0036.

---

## 2. Reused vs New Interfaces

### Reused Interfaces
- **`aiosh-core::service`**: Core data models (`ServiceSpec`, `ServiceStatus`, `ServiceState`, `ServiceStartupMode`, `ServiceAction`, `ActionReport`, `ServiceQuery`), validation rules (`validate_service_name`, `validate_service_spec`), and cycle detection algorithms.
- **`aiosh-core::service_service`**: `ServiceStore` storage abstraction, in-memory status registry, and PID-safe atomic serialization (`save_to_path`).
- **`aiosh-mcp::dispatch`**: `recorded_call` dispatch wrapper executing PEP checks and structured audit emission to `AuditRing`.
- **`aiosh-mcp::pep`**: Policy Enforcement Point capability token validation (`grant_id`).

### New / Extended AIOS MCP Interfaces
- Standardized tool names: `aios.service.validate`, `aios.service.list`, `aios.service.get`, `aios.service.action`, `aios.service.order`.
- JSON-RPC error envelope conventions with deterministic error string patterns.

---

## 3. Tool Specifications

### 3.1 `aios.service.validate`
- **Description**: Validates service name syntax (SS1) or full `ServiceSpec` object against `SS1..SS5` invariants.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Service name to validate against SS1 syntax"
      },
      "spec": {
        "type": "object",
        "description": "Complete ServiceSpec object to validate against SS1..SS5 invariants"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "additionalProperties": false
  }
  ```
- **Validation Rules**:
  - Exactly one of `name` or `spec` must be non-null.
  - If `name` is provided: length <= 128, no ASCII control characters (`c.is_control()`), must match regex `^[a-zA-Z0-9][a-zA-Z0-9_\-\.]{0,126}\.service$`.
  - If `spec` is provided: parsed into `ServiceSpec` and validated against SS1 (name), SS2 (executable paths), SS3 (timeouts), SS4 (restart policy & type), SS5 (no self-dependency).
- **Happy Path Output**:
  ```json
  {
    "ok": true,
    "tool": "aios.service.validate",
    "valid": true,
    "name": "auditd.service",
    "spec": { ... } // included if spec was validated
  }
  ```
- **Failure Path Output**:
  ```json
  {
    "ok": false,
    "error": "Invalid service name: exceeds 128 chars or contains control characters",
    "code": "INVALID_ARGUMENT"
  }
  ```
- **Persistence Effects**: None (pure validation).
- **Audit Effects**: Emits an audit row with action `aios.service.validate`.

---

### 3.2 `aios.service.list`
- **Description**: Lists registered system services with optional pattern, state, or startup mode filtering.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "pattern": {
        "type": "string",
        "description": "Optional substring match on service name and description"
      },
      "state": {
        "type": "string",
        "enum": ["active", "inactive", "activating", "deactivating", "failed", "reloading"],
        "description": "Optional service state filter"
      },
      "startup_mode": {
        "type": "string",
        "enum": ["enabled", "disabled", "static", "masked"],
        "description": "Optional startup mode filter"
      },
      "limit": {
        "type": "integer",
        "description": "Optional limit on returned results count"
      },
      "store_path": {
        "type": "string",
        "description": "Optional path to custom service_store.json"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "additionalProperties": false
  }
  ```
- **Validation Rules**:
  - `pattern`: length <= 256, no control characters.
  - `store_path`: length <= 1024, no control characters.
  - `limit`: non-negative integer.
- **Happy Path Output**:
  ```json
  {
    "ok": true,
    "tool": "aios.service.list",
    "count": 6,
    "services": [
      {
        "name": "auditd.service",
        "description": "AIOS Security Audit Logging Daemon",
        "service_type": "simple",
        "restart_policy": "always",
        "startup_mode": "enabled",
        ...
      }
    ]
  }
  ```
- **Failure Path Output**:
  ```json
  {
    "ok": false,
    "error": "unknown service state 'bogus'",
    "code": "INVALID_ARGUMENT"
  }
  ```
- **Persistence Effects**: None (read-only query).
- **Audit Effects**: Emits an audit row with action `aios.service.list`.

---

### 3.3 `aios.service.get`
- **Description**: Retrieves detailed specification and runtime status of a service by canonical name.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Canonical service name (e.g., auditd.service)"
      },
      "store_path": {
        "type": "string",
        "description": "Optional path to custom service_store.json"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "required": ["name"],
    "additionalProperties": false
  }
  ```
- **Validation Rules**:
  - `name`: non-empty, length <= 128, no control characters.
  - `store_path`: length <= 1024, no control characters.
- **Happy Path Output**:
  ```json
  {
    "ok": true,
    "tool": "aios.service.get",
    "name": "auditd.service",
    "service": {
      "name": "auditd.service",
      "description": "AIOS Security Audit Logging Daemon",
      "exec_start": "/usr/bin/aios-auditd",
      "service_type": "simple",
      "restart_policy": "always",
      "startup_mode": "enabled",
      ...
    },
    "status": {
      "name": "auditd.service",
      "state": "active",
      "pid": 102,
      "exit_code": null,
      "restart_count": 0,
      "last_transition": "2026-09-06T10:00:00Z"
    }
  }
  ```
- **Failure Path Output**:
  ```json
  {
    "ok": false,
    "error": "service 'nonexistent.service' not found in store",
    "code": "NOT_FOUND"
  }
  ```
- **Persistence Effects**: None.
- **Audit Effects**: Emits audit row with action `aios.service.get` and target `name`.

---

### 3.4 `aios.service.action`
- **Description**: Executes a lifecycle action against a registered service.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Target service name"
      },
      "action": {
        "type": "string",
        "enum": ["start", "stop", "restart", "reload", "enable", "disable", "mask", "unmask"],
        "description": "Lifecycle action to perform"
      },
      "store_path": {
        "type": "string",
        "description": "Optional path to custom service_store.json"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "required": ["name", "action"],
    "additionalProperties": false
  }
  ```
- **Validation Rules**:
  - `name`: non-empty, length <= 128, no control characters.
  - `action`: must match one of the 8 canonical action strings (case-insensitive in handler).
  - Masked State Invariant: Activation (`start`, `restart`, `reload`, `enable`) of a `masked` service must be rejected with an error.
- **Happy Path Output**:
  ```json
  {
    "ok": true,
    "tool": "aios.service.action",
    "report": {
      "service_name": "auditd.service",
      "action": "stop",
      "success": true,
      "previous_state": "active",
      "new_state": "inactive",
      "message": "Service auditd.service stopped successfully"
    }
  }
  ```
- **Failure Path Output**:
  ```json
  {
    "ok": false,
    "error": "cannot start service 'example.service': service is masked",
    "code": "OPERATION_FAILED"
  }
  ```
- **Persistence Effects**: If `store_path` is provided, mutations are atomically flushed to disk via `store.save_to_path` using a PID-tagged temp file and un-link cleanup.
- **Audit Effects**: Emits consequential audit row recording `target_entity: name`, `action: aios.service.action`, actor credentials, and grant ID.

---

### 3.5 `aios.service.order`
- **Description**: Computes topological activation sequence for a service and its required dependencies.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Target service name"
      },
      "store_path": {
        "type": "string",
        "description": "Optional path to custom service_store.json"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "required": ["name"],
    "additionalProperties": false
  }
  ```
- **Validation Rules**:
  - `name`: non-empty, length <= 128, no control characters.
  - Cycle detection: Dependency graph must be a directed acyclic graph (DAG). Cyclic dependencies return a cycle error.
- **Happy Path Output**:
  ```json
  {
    "ok": true,
    "tool": "aios.service.order",
    "target": "aios-securityd.service",
    "order": [
      "auditd.service",
      "aios-keystored.service",
      "aios-securityd.service"
    ]
  }
  ```
- **Failure Path Output**:
  ```json
  {
    "ok": false,
    "error": "dependency cycle detected involving service 'svc-a.service'",
    "code": "CYCLE_DETECTED"
  }
  ```
- **Persistence Effects**: None.
- **Audit Effects**: Emits audit row with action `aios.service.order` and target `name`.

---

## 4. Error Handling & Invariants
1. **JSON Envelope Uniformity**: All tool responses return valid JSON. If an operation succeeds, `"ok": true`. If an operation fails, `"ok": false` and `"error": "<diagnostic message>"`.
2. **Never Panic**: Handlers must never panic or unwrap unchecked `Option`/`Result`. All errors must map into `Result<Value, String>`.
3. **No Partial Disk Writes**: File saves utilize temp file creation with PID suffix, synchronization, and atomic rename. Any failure during save unlinks the temporary file.
4. **Honest Audit Trails**: As required by ADR-0035 §F-2, all failures, authorization rejections, and successful calls are logged to the audit ring.
