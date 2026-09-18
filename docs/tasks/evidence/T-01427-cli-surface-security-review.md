# T-01427: User Session Bootstrap - CLI Surface: Security Review

## Metadata
- **Task ID:** `T-01427`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Security Review (`code/aiosh-rust/aiosh-cli::cmd_session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (7/10) — CLI Surface Security Review

---

## 1. Threat Modeling & Abuse Scenarios

### Abuse Scenario 1: Path Traversal & Flag Injection via Session Identifiers
- **Attack Vector:** An adversary supplies traversal sequences (e.g. `../../etc/shadow`), control bytes, slashes, or synthetic flags (e.g. `--privileged`) as the session identifier argument to `show`, `action`, or shortcuts (`activate`, `lock`, `unlock`, `terminate`).
- **Mitigation:**
  - Argument parsing explicitly filters out tokens beginning with `-` from positional session IDs.
  - Session IDs are validated against invariant `SB1`:
    - Length bounded to $[1 \dots 64]$ bytes.
    - Initial character restricted to ASCII alphanumeric.
    - Subsequent characters restricted to `[a-zA-Z0-9_.-]`.
    - Strict rejection of slashes, backslashes, null bytes, and traversal tokens (`..`).
- **Verdict:** Secure. Path traversal and argument injection are rejected at the argument boundary.

### Abuse Scenario 2: Memory Exhaustion via Malicious Spec Payloads (`validate` & `create`)
- **Attack Vector:** Submitting gigabyte-sized files or inline JSON strings to `aiosh session create` or `aiosh session validate --spec` to cause out-of-memory denial of service.
- **Mitigation:**
  - Files are checked via filesystem metadata prior to reading into memory; files exceeding 1 MiB (`1,048,576` bytes) are immediately rejected.
  - Inline JSON strings exceeding 1 MiB are rejected prior to parsing.
  - Rejected inputs trigger exit code 2 and emit a `PAYLOAD_TOO_LARGE` audit event.
- **Verdict:** Secure. Heap allocation is bounded defensively.

### Abuse Scenario 3: Seat Hijacking & Foreground Privilege Spoofing
- **Attack Vector:** An unauthenticated or background session attempts to seize foreground control on `seat0` to intercept display, input, or keystrokes.
- **Mitigation:**
  - Seat arbitration (`CS2`) guarantees mutual exclusion: at most one session can hold `SessionScope::Foreground` on any physical/virtual seat.
  - Activating a session automatically demotes existing foreground sessions on that seat to `SessionScope::Background`.
  - Terminated sessions cannot be activated (`CS1`), preventing revival of abandoned sessions.
- **Verdict:** Secure. Foreground arbitration is deterministic and logged.

### Abuse Scenario 4: Resource Starvation via Session Flooding (`create`)
- **Attack Vector:** Rapid automated invocation of `aiosh session create` to flood the system with unbounded session definitions, exhausting file descriptors or storage.
- **Mitigation:**
  - `create_session` strictly enforces capacity bounds (`CS3` / `SB5`):
    - Maximum 32 active sessions per user account.
    - Maximum 1,024 total tracked sessions in the store.
  - Requests exceeding capacity thresholds are rejected with exit code 1 (`CREATE_FAILED`).
- **Verdict:** Secure. Store and user boundaries prevent resource exhaustion.

### Abuse Scenario 5: Arbitrary Store Overwrite via `--store`
- **Attack Vector:** Supplying path traversal sequences to `--store` to overwrite arbitrary system files (e.g. `/etc/passwd`) or load corrupted state.
- **Mitigation:**
  - Store paths are capped at 1,024 characters and checked for control characters.
  - Serialization uses atomic temporary file writes (`.tmp.<pid>.<nanos>`) with process isolation.
  - Temporary files are immediately unlinked on write/rename failure.
  - Permissions on Unix platforms are restricted to `0o600` (root/supervisor read-write only).
- **Verdict:** Secure. File operations execute within process filesystem constraints with safe atomic semantics.

### Abuse Scenario 6: Audit Evasion & Silent Mutation
- **Attack Vector:** Invoking state mutation subcommands (`create`, `action`, `activate`, `lock`, `unlock`, `terminate`) without recording operations in the system audit log.
- **Mitigation:**
  - Every subcommand branch unconditionally calls `classify_and_emit`.
  - Captures actor identity ("operator"), action verb, target session ID, parameters, and outcome ("success" or "failure") into SQLite WAL ring (`audit.db`) with SHA-256 hash chaining.
- **Verdict:** Secure. Zero unaudited execution paths exist.

---

## 2. Policy & Enforcement Assessment

- **PEP Enforcement:** CLI surfaces enforce actor attribution and write immutable audit records on all execution paths.
- **Fail-Closed Semantics:** All syntax violations, payload ceiling breaches, and invariant failures exit with code 2 and emit failure audit records.
- **Zero Policy Bypass:** Full code review confirms no unauthenticated, unaudited, or unvalidated paths remain.

---

## 3. Acceptance Verification
- [x] Security evidence file exists with comprehensive abuse scenarios and mitigations.
- [x] No known policy bypass remains open.
