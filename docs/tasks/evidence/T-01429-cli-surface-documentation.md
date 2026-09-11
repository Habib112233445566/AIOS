# T-01429: User Session Bootstrap - CLI Surface: Documentation

## Metadata
- **Task ID:** `T-01429`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Documentation (`code/aiosh-cli/README.md`, `code/aiosh-rust/aiosh-cli`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (9/10) — CLI Surface Documentation

---

## 1. Documentation Shipped

### 1.1 CLI Surface Documentation (`code/aiosh-cli/README.md`)
Updated `code/aiosh-cli/README.md` with the complete specification and operational reference for `aiosh session`:
- **Subcommands Index**: Table detailing all primary and shortcut subcommands (`validate`, `list`, `show`, `status`, `action`, `activate`, `lock`, `unlock`, `terminate`, `auth`, `create`).
- **Standardized Invocation Patterns**: Syntax patterns for parameter flags (`--id`, `--user`, `--spec`, `--state`, `--type`, `--seat`, `--limit`, `--store`, `--reason`, `--json`).
- **Machine-Readable Result Envelopes**: Documented standard JSON response envelope and error object format conforming to ADR-0035 §D-2.

---

## 2. CLI Subcommands & Syntax Reference

| Subcommand | Description | Syntax / Options |
|---|---|---|
| `aiosh session validate` | Validate session identifier, user, or specification payload | `aiosh session validate [--id <id>] [--user <name>] [--spec <file_or_json>] [--json]` |
| `aiosh session list` | List tracked sessions with optional filtering | `aiosh session list [--user <name>] [--state <state>] [--type <type>] [--seat <seat>] [--limit <n>] [--store <path>] [--json]` |
| `aiosh session show` | Inspect specification, runtime state, and audit status | `aiosh session show <session_id> [--store <path>] [--json]` |
| `aiosh session status` | Alias for `show` with identical parameters | `aiosh session status <session_id> [--store <path>] [--json]` |
| `aiosh session action` | Execute state transition action | `aiosh session action <session_id> <action> [--reason <reason>] [--store <path>] [--json]` |
| `aiosh session activate` | Shortcut to activate session onto seat | `aiosh session activate <session_id> [--store <path>] [--json]` |
| `aiosh session lock` | Shortcut to lock session display/input | `aiosh session lock <session_id> [--store <path>] [--json]` |
| `aiosh session unlock` | Shortcut to unlock session | `aiosh session unlock <session_id> [--store <path>] [--json]` |
| `aiosh session terminate` | Shortcut to terminate session | `aiosh session terminate <session_id> [--reason <reason>] [--store <path>] [--json]` |
| `aiosh session auth` | Shortcut to mark session authenticated | `aiosh session auth <session_id> [--store <path>] [--json]` |
| `aiosh session create` | Ingest specification and bootstrap new session | `aiosh session create <spec_file_or_json> [--store <path>] [--json]` |

---

## 3. Copy-Pasteable Invocations for Operators and Agents

### 3.1 Validate Session Spec File
```bash
aiosh session validate --spec /etc/aios/sessions/user-kali.json --json
```

### 3.2 List Active Sessions on Seat `seat0`
```bash
aiosh session list --state active --seat seat0 --json
```

### 3.3 Query Canonical Greeter Status
```bash
aiosh session status greeter-seat0 --json
```

### 3.4 Bootstrap New Session from Inline JSON Spec
```bash
aiosh session create '{"session_id":"sess-user1","username":"user1","uid":1000,"gid":1000,"session_type":"wayland","session_class":"user","seat":"seat0","vtnr":1,"display":":0","remote_host":null,"environment":{"XDG_RUNTIME_DIR":"/run/user/1000"}}' --json
```

### 3.5 Activate Session onto Seat
```bash
aiosh session activate sess-user1 --json
```

### 3.6 Lock and Unlock Session
```bash
aiosh session lock sess-user1 --json
aiosh session unlock sess-user1 --json
```

### 3.7 Terminate Session with Audit Reason
```bash
aiosh session terminate sess-user1 --reason "operator logout" --json
```

---

## 4. Response Envelopes & Error Code Mapping

### 4.1 Success Envelope
```json
{
  "code": 0,
  "data": {
    "session_id": "sess-user1",
    "state": "Active",
    "scope": "Foreground",
    "seat": "seat0",
    "locked": false
  },
  "error": null
}
```

### 4.2 Error Envelope
```json
{
  "code": 2,
  "data": null,
  "error": {
    "code": "PAYLOAD_TOO_LARGE",
    "message": "Spec file or content exceeds 1 MiB limit (1048576 bytes)"
  }
}
```

---

## 5. Constraints and Known Limitations (Honest Disclosure)

1. **1 MiB Specification Payload Bound**:
   - Both inline strings and file paths supplied to `--spec` or `create` are rejected with exit code 2 and `PAYLOAD_TOO_LARGE` if size exceeds 1,048,576 bytes.
2. **Store Path String Bounds**:
   - `--store <path>` is checked for length ($\le 1,024$ bytes) and rejection of ASCII control characters (`c.is_control()`). Paths violating these invariants return exit code 2 (`INVALID_ARGUMENT`).
3. **Bounded Query Limits**:
   - `--limit <n>` is strictly constrained to integers in range $[1 \dots 10,000]$. Out-of-bounds or invalid strings return exit code 2 (`INVALID_ARGUMENT`).
4. **Seat Foreground Mutual Exclusion (`CS2`)**:
   - Only one session can occupy the foreground on a given seat at any time. Calling `activate` demotes prior foreground sessions on that seat to background.
5. **Terminal Sink Immutability (`CS1`)**:
   - A session in `Terminated` state cannot undergo any further state transitions. Attempts to do so return error code `ACTION_FAILED`.
6. **Store and User Capacities (`CS3`, `SB5`)**:
   - Maximum 32 active sessions per user account.
   - Maximum 1,024 total tracked sessions in a single store.
7. **Atomic Persistence & Lock Invariants (`CS5`)**:
   - State updates persist via PID+nanosecond temporary files with bounded retries and cleanup on error.
   - Active state strictly entails `locked == false`; Locked state strictly entails `locked == true`.
8. **Non-Repudiation Audit Trail (ADR-0035 §F-2)**:
   - All invocations—including invalid arguments, payload breaches, or unknown commands—generate an immutable SHA-256 hash-chained entry in SQLite WAL audit log (`$AIOSH_HOME/audit.db`).

---

## 6. Related Task Evidence Links
- [T-01421: CLI Surface Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01421-cli-surface-research.md)
- [T-01422: CLI Surface Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01422-cli-surface-specification.md)
- [T-01423: CLI Surface Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01423-cli-surface-scaffold.md)
- [T-01424: CLI Surface Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01424-cli-surface-implementation.md)
- [T-01425: CLI Surface Unit Test](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01425-cli-surface-unit-test.md)
- [T-01426: CLI Surface Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01426-cli-surface-integration.md)
- [T-01427: CLI Surface Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01427-cli-surface-security-review.md)
- [T-01428: CLI Surface Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01428-cli-surface-hardening.md)

---

## 7. Acceptance Verification
- [x] Documentation updated in `code/aiosh-cli/README.md` with working example invocations and syntax tables.
- [x] Constraints and known limitations are explicitly and honestly documented.
- [x] Cross-references to preceding evidence files linked.
