# T-01439: User Session Bootstrap - MCP/API Surface: Documentation

## Metadata
- **Task ID:** `T-01439`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Documentation (`code/aiosh-mcp/README.md`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (9/10) — MCP/API Surface Documentation

---

## 1. Documentation Updates Completed

The documentation for the User Session Bootstrap MCP API surface has been fully incorporated into `code/aiosh-mcp/README.md`.

### 1.1 Tool Manifest Table
The MCP tools reference table was expanded to include `aios.session.create`:
- `aios.session.validate`: Validates session ID, username syntax (SB1, SB2) or full `UserSessionSpec` against invariants (SB1..SB5).
- `aios.session.list`: Lists active/tracked user sessions with optional user, state, type, seat, limit filtering.
- `aios.session.get`: Retrieves runtime status and specification of a session by ID.
- `aios.session.action`: Executes lifecycle action (`authenticate`, `activate`, `lock`, `unlock`, `terminate`) with seat arbitration.
- `aios.session.create`: Atomically provisions a new user or agent session with validation and capacity enforcement.

---

## 2. Copy-Pasteable Tool Call Examples

All 5 tools include copy-pasteable standard JSON-RPC 2.0 requests in `code/aiosh-mcp/README.md`:

### 2.1 Validate Session Specification (`aios.session.validate`)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.session.validate",
    "arguments": {
      "spec": {
        "session_id": "sess-01",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "wayland",
        "session_class": "user",
        "seat": "seat0",
        "vtnr": 7,
        "display": ":0",
        "remote_host": null,
        "environment": {
          "XDG_RUNTIME_DIR": "/run/user/1000",
          "DISPLAY": ":0"
        }
      }
    }
  }
}
```

### 2.2 Query Tracked Sessions (`aios.session.list`)
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.session.list",
    "arguments": {
      "state": "active",
      "seat": "seat0",
      "limit": 50
    }
  }
}
```

### 2.3 Inspect Session Status & Spec (`aios.session.get`)
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.session.get",
    "arguments": {
      "session_id": "greeter-seat0"
    }
  }
}
```

### 2.4 Execute Lifecycle Action (`aios.session.action`)
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "aios.session.action",
    "arguments": {
      "session_id": "sess-01",
      "action": "activate"
    }
  }
}
```

### 2.5 Atomically Provision Session (`aios.session.create`)
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "aios.session.create",
    "arguments": {
      "spec": {
        "session_id": "sess-agent-01",
        "username": "aios-agent",
        "uid": 1001,
        "gid": 1001,
        "session_type": "ai_agent",
        "session_class": "agent",
        "seat": "seat0",
        "vtnr": null,
        "display": null,
        "remote_host": null,
        "environment": {
          "AIOS_AGENT_MODE": "autonomous",
          "AIOS_AGENT_ID": "agent-copilot"
        }
      },
      "store_path": "/tmp/.aios/sessions.json"
    }
  }
}
```

---

## 3. Invariants & Honest Known Limitations

Recorded explicitly in `code/aiosh-mcp/README.md`:
1. **Seat Mutual Exclusion (`CS2`)**: At most one session can hold `SessionScope::Foreground` on any physical/virtual seat (e.g. `seat0`). Activating a session automatically demotes prior foreground sessions on that seat to `SessionScope::Background`.
2. **Two-Stage Teardown & Permanent Termination (`CS1`)**: Active or Locked sessions require a two-stage shutdown (`Active` $\to$ `Terminating` $\to$ `Terminated`). Once in `Terminated`, a session cannot be revived or modified; subsequent actions return an error envelope.
3. **Capacities (`CS3`, `SB5`)**: Maximum 32 active sessions per individual user account; maximum 1,024 total tracked sessions in the store.
4. **Lock Invariant (`CS5`)**: Session `state == Locked` strictly matches `locked == true`; `state == Active` strictly matches `locked == false`. Activity updates never unlock a session.
5. **Hardening Boundaries**:
   - Transport request line ceiling: 1 MiB (`1,048,576` bytes).
   - Ingestion payload ceiling: 1 MiB limit on inline JSON strings/objects in `validate` and `create`.
   - Query limits: `limit` parameter in `aios.session.list` strictly enforced to $[1 \dots 10,000]$.
   - Path sanitization: `store_path` parameter restricted to $\le 1024$ characters with zero control characters.
   - Session identifiers: strictly validated against `SB1` ($[1 \dots 64]$ chars, alphanumeric start, no slashes or traversal sequences).
   - Atomic persistence: 10 MiB store ceiling, atomic write via PID-isolated temporary files (`.tmp.<pid>.<nanos>`), and guaranteed tempfile cleanup on write/rename failure.

---

## 4. Evidence References

Linked directly in `code/aiosh-mcp/README.md`:
- [T-01431 Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01431-mcp-api-surface-research.md)
- [T-01432 Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01432-mcp-api-surface-specification.md)
- [T-01433 Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01433-mcp-api-surface-scaffold.md)
- [T-01434 Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01434-mcp-api-surface-implementation.md)
- [T-01435 Unit Test](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01435-mcp-api-surface-unit-test.md)
- [T-01436 Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01436-mcp-api-surface-integration.md)
- [T-01437 Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01437-mcp-api-surface-security-review.md)
- [T-01438 Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01438-mcp-api-surface-hardening.md)
- [T-01439 Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01439-mcp-api-surface-documentation.md)

---

## 5. Acceptance Verification

- [x] Documentation in `code/aiosh-mcp/README.md` updated with all 5 `aios.session.*` tools.
- [x] Copy-pasteable JSON-RPC 2.0 tool call examples provided.
- [x] Invariants, capacity limits, and hardening constraints stated honestly.
- [x] Direct markdown links to all task evidence files included.
