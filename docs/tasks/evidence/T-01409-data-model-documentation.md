# T-01409: User Session Bootstrap - Data Model: Documentation

## Metadata
- **Task ID:** `T-01409`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Data Model Documentation (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (9/10) — Data Model Documentation

---

## 1. Summary of Delivered Capabilities

The User Session Bootstrap Data Model establishes canonical representations, validation logic, lifecycle state machines, and capacity boundaries for user and agent sessions in AIOS:

1. **Core Data Structures (`code/aiosh-rust/aiosh-core/src/session.rs`)**:
   - `SessionType`: Session environment architecture (`Tty`, `X11`, `Wayland`, `AiAgent`).
   - `SessionClass`: Functional role classification (`User`, `Greeter`, `LockScreen`, `Background`, `Agent`).
   - `SessionState`: Runtime state machine (`Initializing`, `Authenticating`, `Active`, `Locked`, `Terminating`, `Terminated`).
   - `SessionScope`: Focus status on seat (`Foreground`, `Background`).
   - `UserSessionSpec`: Complete declarative configuration for session initialization (session ID, username, UID, GID, type, class, seat, VTNR, display, remote host, environment variables).
   - `UserSessionStatus`: Runtime status tracking session state, leader PID, ISO-8601 timestamps, idle seconds, and locked boolean.
   - `UserSessionAction`: Administrative transitions (`Create`, `Authenticate`, `Activate`, `Lock`, `Unlock`, `Terminate`).
   - `UserSessionQuery`: Multi-attribute search criteria across username, state, type, seat, and count limits.
   - `UserSessionStore`: Canonical in-memory registry and atomic disk persistence (`/run/aios/sessions.json`) with capacity bounding.

2. **Validation Invariants (`SB1..SB5`)**:
   - `SB1`: Session ID syntax: non-empty, $[1 \dots 64]$ chars, alphanumeric start, `[a-zA-Z0-9_.-]`, no traversal (`..`), whitespace, slashes, or shell metacharacters.
   - `SB2`: User identity: POSIX username $[1 \dots 32]$ chars (lowercase/underscore start, `[a-z0-9_-]`), valid UID/GID $\le 2,147,483,647$.
   - `SB3`: State machine transitions: deterministic transition matrix; invalid actions or actions on terminated sessions rejected.
   - `SB4`: Environment isolation: $\le 256$ keys, uppercase alphanumeric syntax, values $\le 4,096$ chars, no null bytes, `XDG_RUNTIME_DIR` absolute Unix path without `..` traversal.
   - `SB5`: Capacity limits: max 32 concurrent active sessions per user, max 1,024 total sessions in store, 10 MiB payload serialization cap.

3. **Operator CLI Surface (`aiosh session validate`)**:
   - `aiosh session validate --id <session_id> [--json]`: Validate session identifier syntax against SB1.
   - `aiosh session validate --user <username> [--json]`: Validate username against SB2.
   - `aiosh session validate --spec <file_or_json> [--json]`: Deep-audit full session specification against SB1..SB5.

4. **Autonomous Agent MCP Tool (`aios.session.validate`)**:
   - Registered in MCP manifest and dispatched via `dispatch::recorded_call` under PEP gating with SQLite WAL audit logging.

---

## 2. Operator CLI Usage Examples

### 1. Validating Session ID Syntax (SB1)
```bash
aiosh session validate --id sess-01
# VALID: Session ID 'sess-01' conforms to SB1 naming syntax

aiosh session validate --id "../evil" --json
```
Output:
```json
{
  "code": 2,
  "data": {
    "session_id": "../evil",
    "valid": false
  },
  "error": {
    "code": "VALIDATION_FAILED",
    "errors": [
      "session ID must start with an ASCII alphanumeric character: '../evil'"
    ],
    "message": "Session ID '../evil' is invalid: session ID must start with an ASCII alphanumeric character: '../evil'"
  }
}
```

### 2. Validating User Identity (SB2)
```bash
aiosh session validate --user kali
# VALID: Username 'kali' conforms to SB2 user identity syntax

aiosh session validate --user "Kali" --json
```
Output:
```json
{
  "code": 2,
  "data": {
    "username": "Kali",
    "valid": false
  },
  "error": {
    "code": "VALIDATION_FAILED",
    "errors": [
      "username must start with a lowercase ASCII letter or underscore: 'Kali'"
    ],
    "message": "Username 'Kali' is invalid: username must start with a lowercase ASCII letter or underscore: 'Kali'"
  }
}
```

### 3. Validating Complete Session Specification (SB1..SB5)
```bash
aiosh session validate --spec '{
  "session_id": "sess-01",
  "username": "kali",
  "uid": 1000,
  "gid": 1000,
  "session_type": "x11",
  "session_class": "user",
  "seat": "seat0",
  "vtnr": 7,
  "display": ":0",
  "remote_host": null,
  "environment": {
    "XDG_RUNTIME_DIR": "/run/user/1000",
    "DISPLAY": ":0"
  }
}' --json
```
Output:
```json
{
  "code": 0,
  "data": {
    "session_id": "sess-01",
    "valid": true,
    "spec": { ... }
  },
  "error": null
}
```

---

## 3. Autonomous Agent MCP Tool Invocations

### Validate Session Identifier
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.session.validate",
    "arguments": {
      "session_id": "sess-01"
    }
  }
}
```

### Validate Session Specification
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.session.validate",
    "arguments": {
      "spec": {
        "session_id": "sess-01",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "x11",
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

---

## 4. Constraints & Known Limitations

1. **Static Validation Scope**: At the data model layer (`T-01401..T-01410`), validation operates statically on in-memory and serialized structures. Live PAM process authentication and D-Bus `systemd-logind` communication are encapsulated in subsequent core service tasks (`T-01411..T-01420`).
2. **Payload Ceilings**: Specification inputs are capped at 1 MiB (`1,048,576` bytes) and store payloads at 10 MiB to prevent memory exhaustion.
3. **Environment Map Restrictions**: Variable keys must be ASCII uppercase or underscore; values cannot contain null bytes.
4. **POSIX Username Requirement**: Usernames must be lowercase or start with underscore, adhering strictly to standard POSIX naming to prevent account spoofing.
5. **Session Capacity**: No single user may exceed 32 concurrent sessions; the store enforces a hard limit of 1,024 total sessions.

---

## 5. Traceability & Task Evidence Links
- Research: [T-01401-data-model-research.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01401-data-model-research.md)
- Specification: [T-01402-data-model-specification.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01402-data-model-specification.md)
- Scaffold: [T-01403-data-model-scaffold.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01403-data-model-scaffold.md)
- Implementation: [T-01404-data-model-implementation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01404-data-model-implementation.md)
- Unit Tests: [T-01405-data-model-unit-test.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01405-data-model-unit-test.md)
- Integration: [T-01406-data-model-integration.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01406-data-model-integration.md)
- Security Review: [T-01407-data-model-security-review.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01407-data-model-security-review.md)
- Hardening: [T-01408-data-model-hardening.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01408-data-model-hardening.md)
- Documentation: [T-01409-data-model-documentation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01409-data-model-documentation.md)

---

## 6. Acceptance Verification
- [x] Documentation updated with working copy-pasteable examples for CLI and MCP.
- [x] Constraints and limitations documented explicitly.
- [x] Evidence links included for complete traceability.
