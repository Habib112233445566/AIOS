# T-01407: User Session Bootstrap - Data Model: Security Review

## Metadata
- **Task ID:** `T-01407`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Data Model Security Review (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (7/10) — Data Model Security Review

---

## 1. Threat Modeling & Abuse Scenarios

### Abuse Scenario AS-01: Session ID Directory Traversal & Injection
- **Attack Vector:** A hostile actor or compromised agent submits a malicious session identifier containing path traversal sequences or shell injection metacharacters (e.g., `../../run/user/root`, `sess-01; rm -rf /`, `sess\0evil`, `sess/01`).
- **Mitigation:** Invariant `SB1` enforced in `validate_session_id`:
  - Length constrained to $[1 \dots 64]$ characters.
  - Leading character must be ASCII alphanumeric `[a-zA-Z0-9]`.
  - Allowed character set restricted to `[a-zA-Z0-9_.-]`.
  - Slashes (`/`, `\`), null bytes (`\0`), directory traversal sequences (`..`), whitespace, and shell metacharacters (`;`, `&`, `|`, `>`, `<`, `$`) are strictly rejected.
- **Verdict:** Secure. Path traversal and injection attacks fail closed before any session record is created or dispatched.

### Abuse Scenario AS-02: Username Spoofing & Malicious Account Injection
- **Attack Vector:** Submitting malformed usernames (e.g. `root\0`, `kali;reboot`, uppercase variants `ROOT` / `Kali` attempting case-folding bypasses, leading digits).
- **Mitigation:** Invariant `SB2` enforced in `validate_username`:
  - Length bounded to $[1 \dots 32]$ characters.
  - Must begin with a lowercase ASCII letter or underscore `[a-z_]`.
  - Allowed characters restricted to lowercase letters, digits, underscores, and hyphens `[a-z0-9_-]`.
  - Uppercase letters, spaces, metacharacters, and null bytes are rejected.
- **Verdict:** Secure. Enforces strict POSIX account identity compliance.

### Abuse Scenario AS-03: Environment Variable Poisoning & XDG Path Traversal
- **Attack Vector:** Injecting hostile environment variables (e.g. `LD_PRELOAD`, `XDG_RUNTIME_DIR = /run/user/../evil`, lowercase keys, values with null bytes).
- **Mitigation:** Invariant `SB4` enforced in `validate_user_session_spec`:
  - Environment map limited to $\le 256$ entries.
  - Keys must be uppercase alphanumeric or underscore `^[A-Z_][A-Z0-9_]{0,63}$`.
  - Values bounded to 4,096 characters with zero null bytes.
  - `XDG_RUNTIME_DIR`, if supplied, must be an absolute path starting with `/` and is strictly checked for directory traversal (`..`).
- **Verdict:** Secure. Prevents library hijacking, path escapes, and environment pollution.

### Abuse Scenario AS-04: Session Flooding & Resource Exhaustion (DoS)
- **Attack Vector:** An attacker continuously creates new sessions to exhaust memory, file descriptors, or process slots (PID exhaustion).
- **Mitigation:** Invariant `SB5` enforced in `UserSessionStore::add_session`:
  - Maximum concurrent sessions per user is capped at $\le 32$. Attempting to create a 33rd active session for the same user is rejected with an explicit error.
  - Maximum total sessions in the store is capped at $\le 1,024$.
  - Deserialization payload is capped at 10 MiB (`MAX_SESSION_STORE_SIZE`).
- **Verdict:** Secure. Enforces hard resource ceilings and prevents state exhaustion attacks.

### Abuse Scenario AS-05: State Machine Hijacking & Unauthorized Unlock
- **Attack Vector:** An attacker attempts illegal state transitions, such as unlocking an unauthenticated or initializing session, or resurrecting a terminated session to bypass authentication.
- **Mitigation:** Invariant `SB3` enforced in `transition_session_state`:
  - Transition matrix strictly defines allowable forward lifecycle paths:
    `Initializing` $\to$ `Authenticating` $\to$ `Active` $\to$ `Locked` $\to$ `Active` $\to$ `Terminating` $\to$ `Terminated`.
  - Illegal transitions (e.g. `Terminated` $\to$ `Active`, `Initializing` $\to$ `Lock`, `Locked` $\to$ `Authenticate`) return an explicit error.
  - Terminated sessions are immutable terminal sinks.
- **Verdict:** Secure. Prevents state confusion, privilege escalation, and auth bypass.

### Abuse Scenario AS-06: Status Contradiction & Telemetry Deception
- **Attack Vector:** Submitting contradictory status telemetry to spoof the security state of a session (e.g. reporting `state == Locked` while claiming `locked == false`, or reporting a terminated session as active in the foreground).
- **Mitigation:** Evaluated in `validate_user_session_status`:
  - Verifies that `state == Locked` requires `locked == true`.
  - Verifies that `state == Active` requires `locked == false`.
  - Verifies that `state == Terminated` cannot be in `Foreground` scope.
- **Verdict:** Secure. Prevents telemetry spoofing and audit misreporting.

---

## 2. Policy Gating (PEP) & Audit Trail Conformance

- **PEP Authorization:** Autonomous agent invocations of `aios.session.validate` pass through `dispatch::recorded_call`, enforcing capability validation via `grant_id`.
- **Immutable Audit Logging:** All operations via CLI (`aiosh session validate`) and MCP emit structured, SHA-256 hash-chained audit rows to SQLite WAL ring buffer (`audit.db`) on both success and failure branches.
- **Policy Bypass Assessment:** Zero unauthenticated or unmonitored paths exist. Fail-closed error handling is enforced across all validation boundaries.

---

## 3. Acceptance Verification
- [x] Comprehensive security review conducted across input validation, path injection, and state handling.
- [x] Threat model and 6 abuse scenarios (AS-01..AS-06) documented with verified mitigations.
- [x] PEP gating and non-repudiation audit logging verified.
- [x] Zero known policy bypasses remain open.
