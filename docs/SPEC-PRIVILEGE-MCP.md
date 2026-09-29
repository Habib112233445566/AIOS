# Specification: Privilege Escalation Prevention MCP/API Surface

## 1. Overview
The Privilege Escalation Prevention MCP/API surface exposes Model Context Protocol (MCP) JSON-RPC 2.0 tools for autonomous agents to inspect, elevate, drop, revoke, and check dynamic privilege execution contexts in `aiosh-mcp`.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `PRIVESC_MCP1` | **Audit Provenance Mandate** | Every privilege MCP tool call is dispatched through `dispatch::recorded_call` and recorded in the audit ring with actor, action, parameters, and outcome. |
| `PRIVESC_MCP2` | **Kernel Immutability** | Any elevation request specifying `SystemKernel` MUST be rejected with error `ERR_PRIVESC_KERNEL_TIER_IMMUTABLE`. |
| `PRIVESC_MCP3` | **Strict Schema Validation** | All tool inputs are strictly validated against JSON Schema Draft-07; unknown fields or boundary violations are rejected. |
| `PRIVESC_MCP4` | **Input Boundary Hardening** | Actor IDs are capped at $\le 128$ bytes, grant tokens at $\le 256$ bytes, capabilities list at $\le 32$ items, with zero control characters permitted. |
| `PRIVESC_MCP5` | **Consistent Envelopes** | Successful tool responses return `{"ok": true, ...}`; errors return explicit standard error strings (`ERR_PRIVESC_*`). |

---

## 3. Tool Definitions & JSON Schemas

### A. Tool: `aios.privilege.status`
- **Description**: Query active privilege context, current tier, baseline tier, grant ID, and held capabilities for an actor.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "actor": { "type": "string", "description": "Actor identifier (defaults to 'mcp-agent')" }
    },
    "additionalProperties": false
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "context": {
      "actor": "mcp-agent",
      "tier": "UserUnprivileged",
      "base_tier": "UserUnprivileged",
      "elevated": false,
      "grant_id": null,
      "capabilities": ["ExecuteBasic"]
    }
  }
  ```

### B. Tool: `aios.privilege.elevate`
- **Description**: Request dynamic privilege elevation to a target tier with grant token validation and capability expansion.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "to": { "type": "string", "description": "Target tier (UserUnprivileged, OperatorStandard, AdminElevated)" },
      "grant": { "type": "string", "description": "Optional grant token, required if target tier is higher than current" },
      "actor": { "type": "string", "description": "Actor identifier (defaults to 'mcp-agent')" },
      "caps": { "type": "array", "items": { "type": "string" }, "description": "Optional capabilities to activate" }
    },
    "required": ["to"],
    "additionalProperties": false
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "context": {
      "actor": "mcp-agent",
      "tier": "AdminElevated",
      "base_tier": "UserUnprivileged",
      "elevated": true,
      "grant_id": "grant-abc",
      "capabilities": ["ExecuteBasic", "ModifySystemConfig"]
    }
  }
  ```

### C. Tool: `aios.privilege.drop`
- **Description**: Voluntarily de-escalate privilege level to a lower tier, shedding higher capabilities and clearing active grants.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "to": { "type": "string", "description": "Target tier to drop to (e.g. UserUnprivileged)" },
      "actor": { "type": "string", "description": "Actor identifier (defaults to 'mcp-agent')" }
    },
    "required": ["to"],
    "additionalProperties": false
  }
  ```

### D. Tool: `aios.privilege.revoke`
- **Description**: Restore baseline privilege level, clearing dynamic grants and revoking elevated capabilities.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "actor": { "type": "string", "description": "Actor identifier (defaults to 'mcp-agent')" }
    },
    "additionalProperties": false
  }
  ```

### E. Tool: `aios.privilege.check`
- **Description**: Verify whether an actor context holds a specific capability.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "cap": { "type": "string", "description": "Capability name to verify" },
      "actor": { "type": "string", "description": "Actor identifier (defaults to 'mcp-agent')" }
    },
    "required": ["cap"],
    "additionalProperties": false
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "actor": "mcp-agent",
    "capability": "AdminControl",
    "held": true
  }
  ```

---

## 4. Error Codes Matrix

| Error Code | HTTP / RPC Semantic | Trigger Condition |
|---|---|---|
| `ERR_PRIVESC_ACTOR_NOT_FOUND` | 404 Not Found | Explicit actor ID not registered in context store |
| `ERR_PRIVESC_INVALID_TIER` | 400 Bad Request | Tier string fails to parse to known `PrivilegeLevel` |
| `ERR_PRIVESC_KERNEL_TIER_IMMUTABLE` | 403 Forbidden | Request attempted elevation to `SystemKernel` |
| `ERR_PRIVESC_GRANT_REQUIRED` | 401 Unauthorized | Elevation requested to higher tier without valid grant |
| `ERR_PRIVESC_INVALID_INPUT` | 400 Bad Request | Bounds violation (string length, control characters, excess caps) |

---

## 5. Storage, Path Traversal & Hardening Bounds

| Parameter / Boundary | Limit | Behavior on Exceeded / Violation |
|---|---|---|
| State store path traversal (`..`) | Forbidden | Discarded; falls back safely to default local state file |
| State store payload size | $\le 1\text{ MiB}$ | Reject load / fallback to clean in-memory state |
| Actor ID Length | $\le 128\text{ bytes}$ | Reject with `ERR_PRIVESC_INVALID_INPUT` |
| Actor ID Characters | No control characters | Reject with `ERR_PRIVESC_INVALID_INPUT` |
| Grant Token Length | $\le 256\text{ bytes}$ | Reject with `ERR_PRIVESC_INVALID_INPUT` |
| Grant Token Characters | No control characters | Reject with `ERR_PRIVESC_INVALID_INPUT` |
| Max Capabilities per Request | $\le 32$ | Reject with `ERR_PRIVESC_INVALID_INPUT` |

---

## 6. Verification & Automated Test Suites
- Unit Tests: `code/aiosh-mcp/tests/test_privilege_mcp.py`
- Integration Tests: `code/aiosh-mcp/tests/test_privilege_automated_smoke.py`

