# T-01417: User Session Bootstrap - Core Service: Security Review

## Metadata
- **Task ID:** `T-01417`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Core Service Security Review (`session_service.rs`, `aiosh-cli`, `aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (7/10) — Core Service Security Review

---

## 1. Security Architecture & Threat Model

The **User Session Bootstrap Core Service** (`UserSessionService`) coordinates session creation, credential binding, seat arbitration, lock screens, and process group termination. As a core OS substrate component, it presents several critical security boundaries:

1. **Input Validation & Sanitization Boundaries**:
   - Session identifiers (`session_id`) passed via CLI `--id` or MCP arguments.
   - User identities (`username`, `uid`, `gid`) specifying process ownership.
   - Seat identifiers (`seat0`, `seat1`) controlling hardware display/input routing.
   - Custom environment variable maps (`environment`) controlling process execution contexts.
2. **Policy Enforcement Point (PEP) Gating**:
   - Every administrative action (`Create`, `Authenticate`, `Activate`, `Lock`, `Unlock`, `Terminate`) must be subject to capability checks.
   - Unauthenticated or unauthorized callers must be blocked prior to state mutation.
3. **Non-Repudiation Audit Logging (ADR-0035 / ADR-0036)**:
   - Consequential state changes must emit structured events into the SQLite WAL audit ring.
   - State-changing operations must log actor, capability grant ID, timestamp, targeted session ID, previous state, new state, and success/error status.

---

## 2. Abuse Scenarios & Mitigations (AS-01..AS-06)

| Scenario ID | Attack Vector / Misuse Description | Potential Impact | Built-in Mitigation Mechanism | Validation Status |
|---|---|---|---|---|
| **AS-01** | **Session ID Path Traversal & Injection**: Malicious input (e.g. `../../etc/shadow`, `/tmp/pwn`, `sess\0evil`) passed as `session_id`. | Arbitrary file overwrite, path escape in persistence, audit log injection. | `validate_session_id` strictly requires $[1 \dots 64]$ chars, alphanumeric start, allowed charset `[a-zA-Z0-9_.-]`, rejects `/`, `\`, `\0`, `..`. MCP and CLI reject control characters. | **VERIFIED (PASS)** |
| **AS-02** | **Terminated Session Resurrection**: Attempting to invoke `Authenticate`, `Activate`, or `Unlock` on a session already in state `Terminated`. | Zombie process persistence, unauthorized re-entry, audit evasion. | `transition_session_state` strictly checks pre-states (`CS1`). Any action on `Terminated` returns an auditable error `Cannot perform action on terminated session`. State transitions are strictly monotonic towards termination. | **VERIFIED (PASS)** |
| **AS-03** | **Seat Hijacking & Multiple Foreground Conflicts**: Attempting to activate multiple sessions simultaneously on `seat0` to intercept input or cause display collision. | Keyboard/mouse input snooping, race condition in display focus. | Atomic seat arbitration (`CS2`) in `apply_action`: Activating a session on seat $S$ automatically demotes all other sessions on seat $S$ from `SessionScope::Foreground` to `SessionScope::Background`. | **VERIFIED (PASS)** |
| **AS-04** | **Resource Exhaustion (Session Flooding DoS)**: Malicious actor creating thousands of dummy sessions to exhaust system memory, file descriptors, or PIDs. | Memory exhaustion, store serialization DOS, PID table exhaustion. | Strict capacity limits (`CS3` / `SB5`): Maximum 32 active sessions per user; maximum 1,024 total sessions in store; 10 MiB store file cap. Rejections occur before allocation. | **VERIFIED (PASS)** |
| **AS-05** | **Lock Screen Bypass via Activity Falsification**: Attempting to call `touch_activity` or `update_idle` on a locked session to restore `Active` state without credentials. | Unauthorized session access bypassing screen lock. | `touch_activity` and `update_idle` only alter monotonic counter `idle_seconds`. They cannot alter `state` or `locked = true`. Unlocking strictly requires `apply_action(id, Unlock)` which verifies state transitions. | **VERIFIED (PASS)** |
| **AS-06** | **Unauthenticated MCP Tool Invocation**: Autonomous agent calling `aios.session.action` without a valid capability grant. | Privilege escalation, unauthorized session termination of operator. | All MCP session tools are routed through `dispatch::recorded_call` which checks `pep.check_grant` and emits audit records with actor attribution. | **VERIFIED (PASS)** |

---

## 3. PEP Gating & Audit Non-Repudiation

### 3.1 CLI Audit Trail Verification
Every execution of `aiosh session action` calls `classify_and_emit` logging:
```json
{
  "subsystem": "session",
  "action": "action",
  "actor": "operator",
  "session_id": "<target_id>",
  "action_type": "<authenticate|activate|lock|unlock|terminate>",
  "previous_state": "<state>",
  "new_state": "<state>",
  "outcome": "success" | "failure"
}
```

### 3.2 MCP Tool PEP Verification
- `aios.session.validate`: Read-only validation, logged via `dispatch::recorded_call`.
- `aios.session.list`: Read-only discovery query, logged via `dispatch::recorded_call`.
- `aios.session.get`: Read-only inspection query, logged via `dispatch::recorded_call`.
- `aios.session.action`: Consequential mutation, requires valid capability grant and produces full SQLite WAL audit row with non-repudiation timestamp.

---

## 4. Security Audit Findings & Resolutions

- **Finding SEC-01 (Store Persistence Path Traversal)**: If `--store` flag is supplied by user, arbitrary filesystem paths could be written.
  - *Resolution*: Path length bounded to $\le 1024$ chars, control characters rejected, and atomic write via temporary file + rename guarantees no partial file overwrite.
- **Finding SEC-02 (Oversized Payloads)**: Inline JSON specs or oversized store files could exhaust memory during parse.
  - *Resolution*: Enforced 1 MiB payload cap on `--spec` inputs and 10 MiB cap on serialized store files.

---

## 5. Acceptance Verification

- [x] Security evidence document exists and documents abuse scenarios AS-01..AS-06.
- [x] No known policy bypasses or unvalidated input paths remain open.
- [x] PEP capability checks and audit-row emission confirmed on all state-changing paths.
