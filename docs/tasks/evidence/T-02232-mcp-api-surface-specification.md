# Task Evidence: T-02232 (Grant Lifecycle / MCP/API surface: Specification)

## 1. Metadata
- **Task ID:** `T-02232`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle MCP/API Surface Specification (`code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic 4: Grant Lifecycle MCP Surface (4/10) — Specification

---

## 2. Interface Contract Specification

### 2.1 Complete MCP Grant Suite Overview

| MCP Tool Name | Action / Purpose | Calling Role | Invariants Enforced |
|:---|:---|:---|:---|
| `aios.pep.grant.issue` | Issue new root authorization grant | Operator / Admin Agent | Idempotency, parameter hygiene, format validation |
| `aios.pep.grant.attenuate` | Derive restricted child grant | Grant Holder | Strict rights subset containment, depth decrement |
| `aios.pep.grant.list` | List grants with optional filtering | Reader / Auditor | State & subject filter constraints |
| `aios.pep.grant.inspect` | Inspect detailed grant attributes | Reader / Auditor | Exact ID lookup, non-existent grant error |
| `aios.pep.grant.validate` | Validate grant eligibility for action | Execution Interceptor | Temporal validity, subject check, right check |
| `aios.pep.grant.revoke` | Revoke grant and cascade to children | Admin / Security Monitor | State transition to Revoked, recursive descendant sweep |
| `aios.pep.grant.sweep` | Transition expired grants to `Expired` | Scheduled Worker | Temporal comparison against current timestamp |

---

### 2.2 Specification for `aios.pep.grant.issue`

#### 2.2.1 Input Schema (`inputSchema`)
```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "description": "Unique identifier for the grant (1-64 alphanumeric, dash, dot, underscore)"
    },
    "issuer": {
      "type": "string",
      "description": "Issuing authority identity (default: 'mcp-agent')"
    },
    "subject": {
      "type": "string",
      "description": "Authorized subject entity (e.g. 'agent:worker')"
    },
    "scope_type": {
      "type": "string",
      "enum": ["filesystem", "network", "process", "ipc", "system", "custom"],
      "description": "Capability resource domain"
    },
    "scope_path": {
      "type": "string",
      "description": "Target resource path or identifier (optional)"
    },
    "rights": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Granted rights: read, write, execute, delete, admin, delegate"
    },
    "delegation_depth": {
      "type": "integer",
      "minimum": 0,
      "description": "Maximum delegation depth (default: 0)"
    },
    "expires_at": {
      "type": "string",
      "description": "RFC 3339 expiration timestamp (optional)"
    },
    "not_before": {
      "type": "string",
      "description": "RFC 3339 not-before timestamp (optional)"
    },
    "max_invocations": {
      "type": "integer",
      "minimum": 1,
      "description": "Maximum permitted invocations quota (optional)"
    },
    "max_bytes": {
      "type": "integer",
      "minimum": 1,
      "description": "Maximum permitted bytes transferred quota (optional)"
    },
    "store_path": {
      "type": "string",
      "description": "Optional path to PEP grants JSON store (default: 'pep_grants.json')"
    },
    "grant_id": {
      "type": "string",
      "description": "Optional caller authorization grant ID"
    }
  },
  "required": ["id", "subject", "scope_type", "rights"],
  "additionalProperties": false
}
```

#### 2.2.2 Output Contract
- **Happy Path Response**:
  ```json
  {
    "ok": true,
    "tool": "aios.pep.grant.issue",
    "grant": {
      "id": "g-root-1",
      "issuer": "mcp-agent",
      "subject": "agent:worker",
      "scope": {
        "type": "filesystem",
        "details": { "path": "/data" }
      },
      "rights": ["read", "write"],
      "state": "active",
      "parent_grant_id": null,
      "constraints": {
        "not_before": null,
        "expires_at": "2026-12-31T23:59:59Z",
        "max_invocations": null,
        "invocations_used": 0,
        "max_bytes": null,
        "bytes_used": 0,
        "max_delegation_depth": 0
      },
      "revocation": null,
      "metadata": {},
      "created_at": "2026-09-23T00:30:00Z",
      "updated_at": "2026-09-23T00:30:00Z"
    }
  }
  ```
- **Failure Responses**:
  - Missing parameter: `{"ok": false, "error": "missing required parameter: <param>"}`
  - Invalid right: `{"ok": false, "error": "unknown capability right: <right>"}`
  - Duplicate ID: `{"ok": false, "error": "grant ID already exists: <id>"}`
  - Path traversal: `{"ok": false, "error": "invalid grant store path: ..."}`

---

## 3. Cryptographic Audit Invariants

Pursuant to ADR-0035 §F-2:
1. Invocations of `aios.pep.grant.issue` route strictly through `dispatch::recorded_call`.
2. Emits an audit row with action `"aios.pep.grant.issue"`, tool `"aios.pep.grant.issue"`, full input parameters, outcome, and SHA-256 previous-hash chain pointer into SQLite `$AIOSH_HOME/audit.db`.

---

## 4. Acceptance Confirmation
- [x] Input schema, output envelopes, and failure conditions fully defined.
- [x] Existing interface reuse (`PepGrantService`, `PepGrantStore`, `dispatch::recorded_call`) established.
- [x] Conforms strictly to MCP stdio protocol and ADR-0035 audit invariants.
