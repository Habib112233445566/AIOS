# T-01419: User Session Bootstrap - Core Service: Documentation

## Metadata
- **Task ID:** `T-01419`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Core Service Documentation (`code/aiosh-mcp/README.md`, `code/aiosh-rust/aiosh-core`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (9/10) — Core Service Documentation

---

## 1. Documentation Shipped

### 1.1 MCP API Surface Documentation (`code/aiosh-mcp/README.md`)
Updated the MCP server specification and tools index with the complete user session tool suite:
- `aios.session.validate`: Syntactic and invariant validation for session IDs, usernames, and complete `UserSessionSpec` objects.
- `aios.session.list`: Querying tracked sessions with filtering by `user`, `state`, `type`, `seat`, and result pagination `limit`.
- `aios.session.get`: Full inspection of session specification and runtime status by `session_id`.
- `aios.session.action`: Lifecycle management (`authenticate`, `activate`, `lock`, `unlock`, `terminate`) with seat arbitration.

### 1.2 Copy-Pasteable Tool Calls & CLI Invocations

#### MCP Tool Call: List Active Sessions on `seat0`
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.session.list",
    "arguments": {
      "state": "active",
      "seat": "seat0"
    }
  }
}
```

#### MCP Tool Call: Inspect Session
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.session.get",
    "arguments": {
      "session_id": "greeter-seat0"
    }
  }
}
```

#### MCP Tool Call: Execute Lifecycle Action
```json
{
  "jsonrpc": "2.0",
  "id": 3,
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

#### Operator CLI Commands
```bash
# Validate user session specification file
aiosh session validate --spec /etc/aios/sessions/user-kali.json --json

# List active sessions on primary seat
aiosh session list --state active --seat seat0 --json

# Display status and spec of canonical greeter
aiosh session show greeter-seat0 --json

# Lock an active session
aiosh session action greeter-seat0 lock --json

# Unlock session
aiosh session action greeter-seat0 unlock --json
```

---

## 2. Constraints and Limitations (Honest Disclosure)

1. **Seat Mutual Exclusion (`CS2`)**: At most one session per seat can hold `SessionScope::Foreground`. When `apply_action(id, Activate)` is executed, any existing foreground session on the target seat is demoted to `SessionScope::Background`.
2. **Terminal State Monotonicity (`CS1`)**: The `Terminated` state is a terminal sink. Once reached, all subsequent lifecycle actions fail with explicit errors.
3. **Hard Capacity Ceilings (`CS3` / `SB5`)**:
   - $\le 32$ concurrently active sessions per user account.
   - $\le 1,024$ total sessions in the store.
4. **Lock Invariant Consistency (`CS5`)**:
   - `SessionState::Locked` strictly corresponds to `locked == true`.
   - `SessionState::Active` strictly corresponds to `locked == false`.
   - Activity updates (`touch_activity`) reset idle duration to 0, but never clear the `locked` flag.
5. **Storage Ceilings**:
   - 10 MiB store limit (`MAX_SESSION_STORE_SIZE`).
   - 1 MiB spec payload cap.

---

## 3. Related Task Evidence Links
- [T-01411: Core Service Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01411-core-service-research.md)
- [T-01412: Core Service Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01412-core-service-specification.md)
- [T-01413: Core Service Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01413-core-service-scaffold.md)
- [T-01414: Core Service Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01414-core-service-implementation.md)
- [T-01415: Core Service Unit Test](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01415-core-service-unit-test.md)
- [T-01416: Core Service Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01416-core-service-integration.md)
- [T-01417: Core Service Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01417-core-service-security-review.md)
- [T-01418: Core Service Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01418-core-service-hardening.md)

---

## 4. Acceptance Verification
- [x] Documentation updated in `code/aiosh-mcp/README.md` with copy-pasteable examples.
- [x] Limitations and constraints clearly recorded.
- [x] Links to preceding evidence documents included.
