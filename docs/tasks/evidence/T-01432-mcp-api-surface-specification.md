# T-01432: User Session Bootstrap - MCP/API Surface: Specification

## Metadata
- **Task ID:** `T-01432`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Specification (`code/aiosh-rust/aiosh-mcp`, `code/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (2/10) — MCP/API Surface Specification

---

## 1. Executive Summary & Architectural Role

The Model Context Protocol (MCP) API surface provides autonomous AI agents (S-rank kernel co-pilots, security automation agents, background daemons) with programmatic, structured, and auditable control over Linux user and agent sessions.

Per **ADR-0035 §D-2**, MCP over standard bidirectional JSON-RPC 2.0 stdio streams is the sole tool execution protocol for AI models in AIOS. Every tool call is intercepted by the Policy Enforcement Point (PEP gate, **ADR-0035 §D-4**) and logged synchronously with SHA-256 hash chaining to the SQLite WAL ring buffer (`audit.db`, **ADR-0035 §F-2**).

---

## 2. MCP Tool Manifest Specifications

The User Session Bootstrap subsystem exposes five authoritative MCP tools under the `aios.session.*` namespace:

### 2.1 `aios.session.validate`
- **Description**: Validate session ID syntax (`SB1`), username (`SB2`), or complete `UserSessionSpec` against invariants `SB1..SB5`.
- **JSON Schema (`inputSchema`)**:
  ```json
  {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session identifier to validate against SB1 syntax"
      },
      "username": {
        "type": "string",
        "description": "Username to validate against SB2 syntax"
      },
      "spec": {
        "type": "object",
        "description": "Complete UserSessionSpec object to validate against SB1..SB5 invariants"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "additionalProperties": false
  }
  ```
- **Inputs**: At least one of `session_id`, `username`, or `spec`.
- **Payload Bound**: Maximum 1,048,576 bytes (1 MiB).
- **Success Response**:
  ```json
  {
    "ok": true,
    "tool": "aios.session.validate",
    "valid": true,
    "session_id": "sess-01"
  }
  ```
- **Error Cases**:
  - All three parameters omitted: returns error with message `"Either 'session_id', 'username', or 'spec' parameter is required"`.
  - Syntax or invariant violation: returns `{ "ok": false, "tool": "aios.session.validate", "error": "..." }`.
- **Side Effects**: Read-only; zero persistent mutations.

---

### 2.2 `aios.session.list`
- **Description**: List tracked user and agent sessions with optional filtering by user, state, session type, seat, and pagination limit.
- **JSON Schema (`inputSchema`)**:
  ```json
  {
    "type": "object",
    "properties": {
      "username": {
        "type": "string",
        "description": "Filter by username (1..32 chars)"
      },
      "state": {
        "type": "string",
        "description": "Filter by session state (initializing, authenticating, active, locked, terminating, terminated)"
      },
      "session_type": {
        "type": "string",
        "description": "Filter by session type (tty, x11, wayland, ai_agent)"
      },
      "seat": {
        "type": "string",
        "description": "Filter by seat (e.g. seat0)"
      },
      "limit": {
        "type": "integer",
        "description": "Maximum number of sessions to return (1..10000)"
      },
      "store_path": {
        "type": "string",
        "description": "Optional path to custom session store JSON file"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "additionalProperties": false
  }
  ```
- **Validation**:
  - `limit` bounded to strictly positive integer: $1 \le \text{limit} \le 10,000$.
  - `store_path` bounded to $\le 1,024$ characters without ASCII control characters.
- **Success Response**:
  ```json
  {
    "ok": true,
    "tool": "aios.session.list",
    "count": 1,
    "sessions": [
      {
        "session_id": "greeter-seat0",
        "username": "lightdm",
        "uid": 110,
        "state": "Active",
        "scope": "Foreground",
        "session_type": "X11",
        "session_class": "Greeter",
        "seat": "seat0",
        "locked": false,
        "idle_seconds": 0
      }
    ]
  }
  ```
- **Error Cases**:
  - Limit out of bounds ($< 1$ or $> 10,000$): returns error.
  - `store_path` unreadable or exceeds length bound: returns error.
- **Side Effects**: Read-only.

---

### 2.3 `aios.session.get`
- **Description**: Retrieve runtime status and complete specification for a designated session identifier.
- **JSON Schema (`inputSchema`)**:
  ```json
  {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Target session identifier (1..64 chars)"
      },
      "store_path": {
        "type": "string",
        "description": "Optional path to custom session store JSON file"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "required": ["session_id"],
    "additionalProperties": false
  }
  ```
- **Validation**:
  - `session_id` must be $[1 \dots 64]$ chars without ASCII control characters.
  - `store_path` bounded to $\le 1,024$ chars without ASCII control characters.
- **Success Response**:
  ```json
  {
    "ok": true,
    "tool": "aios.session.get",
    "session_id": "greeter-seat0",
    "status": {
      "session_id": "greeter-seat0",
      "username": "lightdm",
      "uid": 110,
      "state": "Active",
      "scope": "Foreground",
      "session_type": "X11",
      "session_class": "Greeter",
      "seat": "seat0",
      "vtnr": 7,
      "display": ":0",
      "leader_pid": null,
      "locked": false,
      "idle_seconds": 0,
      "created_at": "2026-09-09T00:00:00Z",
      "last_active_at": "2026-09-09T00:00:00Z"
    },
    "spec": { ... }
  }
  ```
- **Error Cases**:
  - Missing `session_id`: returns `"Missing required 'session_id'"`.
  - Session does not exist in store: returns `"Session '<id>' not found"`.
- **Side Effects**: Read-only.

---

### 2.4 `aios.session.action`
- **Description**: Execute a lifecycle transition action (`authenticate`, `activate`, `lock`, `unlock`, `terminate`) on an existing session, enforcing FSM monotonicity (`CS1`) and seat mutual exclusion (`CS2`).
- **JSON Schema (`inputSchema`)**:
  ```json
  {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Target session identifier (1..64 chars)"
      },
      "action": {
        "type": "string",
        "description": "Lifecycle action to execute: authenticate, activate, lock, unlock, terminate"
      },
      "reason": {
        "type": "string",
        "description": "Optional administrative reason for action (1..512 chars)"
      },
      "store_path": {
        "type": "string",
        "description": "Optional path to custom session store JSON file"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "required": ["session_id", "action"],
    "additionalProperties": false
  }
  ```
- **Validation**:
  - `action` must match one of the 5 canonical verbs.
  - `session_id` bounded to $[1 \dots 64]$ chars without control characters.
  - `reason` bounded to $\le 512$ characters.
- **Success Response**:
  ```json
  {
    "ok": true,
    "tool": "aios.session.action",
    "report": {
      "session_id": "sess-user1",
      "action": "Activate",
      "previous_state": "Authenticating",
      "new_state": "Active",
      "success": true,
      "error": null,
      "timestamp": "2026-09-09T23:45:00Z"
    }
  }
  ```
- **Error Cases**:
  - Invalid state transition (e.g., activating a `Terminated` session): returns error with details from `UserSessionService`.
  - Non-existent session: returns session not found.
- **Side Effects**: Mutates session in runtime store; persists atomically to disk when `store_path` is specified.

---

### 2.5 `aios.session.create` (New AIOS Tool)
- **Description**: Provision and register a new user or autonomous agent session from a specification payload, evaluating invariant rules and capacity limits.
- **JSON Schema (`inputSchema`)**:
  ```json
  {
    "type": "object",
    "properties": {
      "spec": {
        "type": "object",
        "description": "Complete UserSessionSpec payload defining session configuration"
      },
      "store_path": {
        "type": "string",
        "description": "Optional path to custom session store JSON file"
      },
      "grant_id": {
        "type": "string",
        "description": "Optional PEP authorization grant ID"
      }
    },
    "required": ["spec"],
    "additionalProperties": false
  }
  ```
- **Validation**:
  - Ingested `spec` object validated against `validate_user_session_spec`.
  - Serialized payload size bounded to $\le 1,048,576$ bytes (1 MiB).
  - User capacity checked ($\le 32$ active sessions per user account).
  - Store capacity checked ($\le 1,024$ total sessions in store).
- **Success Response**:
  ```json
  {
    "ok": true,
    "tool": "aios.session.create",
    "session_id": "agent-co-pilot-01",
    "spec": { ... },
    "status": {
      "session_id": "agent-co-pilot-01",
      "username": "kali",
      "uid": 1000,
      "state": "Initializing",
      "scope": "Background",
      "session_type": "AiAgent",
      "session_class": "Agent",
      "seat": "seat0",
      "locked": false,
      "created_at": "2026-09-09T23:45:00Z"
    }
  }
  ```
- **Error Cases**:
  - Missing `spec`: returns `"Missing required 'spec' parameter"`.
  - Capacity exceeded: returns `"User '<username>' has reached maximum active session limit"`.
  - Duplicate session ID: returns `"Session '<id>' already exists"`.
  - Invariant violation: returns detailed invariant violation error.
- **Side Effects**: Inserts session into store; atomically persists to disk when `store_path` is supplied.

---

## 3. Reuse vs. AIOS-Specific Elements

### Reused Upstream Interfaces
- **Anthropic MCP Specification (2024-11-05)**: Standard tool discovery (`tools/list`), argument envelope passing, and JSON Schema definitions.
- **systemd-logind Semantics**: Standard session lifecycle actions (`authenticate`, `activate`, `lock`, `unlock`, `terminate`), VT assignment, and seat mutual exclusion.
- **Rust Subsystem Modules**: `aiosh-core::session` and `session_service`.

### AIOS-Specific Extensions
- **`aios.session.create`**: Direct programmatic session bootstrap for autonomous AI agents, eliminating manual file writes or shell script execution.
- **`reason` Field in `aios.session.action`**: Auditable administrative context captured directly into SQLite WAL ring buffer.
- **Non-Repudiation Audit Record (`ADR-0035 §F-2`)**: Synchronous SHA-256 hash-chained event appended to `$AIOSH_HOME/audit.db` on every invocation branch.
- **First-Class AI Agent Support**: `session_type: "ai_agent"` and `session_class: "agent"` enabling native co-pilot governance.

---

## 4. Audit & Non-Repudiation Contracts

Every tool call dispatched through `code/aiosh-rust/aiosh-mcp/src/main.rs` invokes `dispatch::recorded_call`:
1. Verifies PEP capability permissions via `pep.check(grant_id, tool_name, target)`.
2. Executes closure.
3. Appends an immutable row to `audit.db` with:
   - `tool`: e.g. `"aios.session.create"` or `"aios.session.action"`
   - `command`: Canonical tool invocation string
   - `args_json`: Canonical JSON of arguments
   - `target`: Target session ID or seat
   - `outcome`: `"ok"` on success, `"refused"` on PEP failure, `"error"` on operational failure
   - `hash`: SHA-256 hash chaining to previous head

---

## 5. Acceptance Criteria
- [x] Full specification of inputs, outputs, error envelopes, and persistence effects for all 5 tools.
- [x] Reused upstream standards and AIOS-specific extensions explicitly distinguished.
- [x] Bounded limits and security invariants formally specified.
