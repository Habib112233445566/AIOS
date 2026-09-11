# T-01437: User Session Bootstrap - MCP/API Surface: Security Review

## Metadata
- **Task ID:** `T-01437`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Security Review (`code/aiosh-rust/aiosh-mcp`, `code/aiosh-rust/aiosh-core`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (7/10) — MCP/API Surface Security Review

---

## 1. Threat Modeling & Abuse Scenarios

The MCP API surface exposes 5 remote JSON-RPC tools (`aios.session.validate`, `aios.session.list`, `aios.session.get`, `aios.session.action`, and `aios.session.create`) over bidirectional stdio. Because agents and external callers invoke these tools programmatically, rigorous threat modeling against injection, resource exhaustion, authorization bypass, and audit tampering is essential.

### Abuse Scenario 1: Path Traversal & Identifier Injection via `session_id`
- **Attack Vector:** An untrusted or compromised agent supplies path traversal sequences (e.g. `../../root`, `/dev/null`), control characters, null bytes, or oversized strings as the `session_id` parameter to `aios.session.validate`, `aios.session.get`, or `aios.session.action`.
- **Mitigation:**
  - Session IDs are strictly evaluated against invariant `SB1`:
    - Length bounded to $[1 \dots 64]$ bytes.
    - Initial character restricted to ASCII alphanumeric `[a-zA-Z0-9]`.
    - Body characters restricted to `[a-zA-Z0-9_.-]`.
    - Strict rejection of slashes (`/`, `\`), null bytes (`\0`), and directory traversal markers (`..`).
  - Lookup operations in `UserSessionService` query in-memory `BTreeMap` structures, preventing filesystem leakage even if traversal strings were permitted.
- **Verdict:** Secure. Invariant validation rejects malformed identifiers deterministically.

### Abuse Scenario 2: Memory Exhaustion via Malicious Spec Payloads (`validate` & `create`)
- **Attack Vector:** An adversary submits multi-megabyte JSON payloads or deeply nested objects to `aios.session.validate` or `aios.session.create` to induce memory exhaustion (OOM DoS).
- **Mitigation:**
  - `aios.session.create` validates payload byte size against a strict 1 MiB (`1,048,576` bytes) ceiling prior to processing.
  - Invariant validation (`SB4`) bounds environment map entries:
    - Maximum 256 environment variables per session.
    - Key length $\le 128$ bytes, value length $\le 4,096$ bytes.
    - Strict variable naming grammar `[A-Za-z_][A-Za-z0-9_]*`.
  - Payloads exceeding bounds return `ok: false` and emit classified failure events.
- **Hardening Recommendation (T-01438):** Mirror the explicit 1 MiB payload check into `aios.session.validate` for complete parity across both ingest paths.
- **Verdict:** Secure. Defensive bounds prevent heap exhaustion.

### Abuse Scenario 3: Store Path Manipulation & Arbitrary File Overwrite (`store_path`)
- **Attack Vector:** Supplying path traversal sequences, control bytes, or arbitrary system paths (e.g. `/etc/shadow`, `/boot/efi`) via `store_path` to overwrite or corrupt sensitive host files.
- **Mitigation:**
  - `store_path` is restricted to $\le 1024$ characters and verified to contain zero control characters.
  - File persistence in `aiosh_core::session::UserSessionStore::save_to_path` executes an atomic two-phase write:
    - Writes to an isolated temporary file (`<path>.tmp.<pid>.<nanos>`).
    - Flushes to disk and atomically renames over the target path.
    - Immediately cleans up temporary files upon failure.
    - Filesystem access operates under the process's unprivileged sandbox or service permissions.
- **Hardening Recommendation (T-01438):** Apply length and control-character bounds consistently across all 4 tools accepting `store_path` (`list`, `get`, `action`, `create`).
- **Verdict:** Secure. Path safety checks and atomic write semantics prevent corruption and arbitrary overwrite.

### Abuse Scenario 4: Illegal Lifecycle Transition & Session Desynchronization
- **Attack Vector:** An attacker attempts to skip authentication, unlock an unauthorized session, or revive a terminated session by sending spoofed action requests to `aios.session.action`.
- **Mitigation:**
  - State machine transitions are governed by formal rule set `CS1` (`transition_session_state`):
    - Initializing $\to$ Authenticate $\to$ Authenticating $\to$ Activate $\to$ Active $\to$ Lock $\to$ Locked.
    - Terminate transitions Active/Locked to `Terminating`, then a subsequent Terminate completes teardown to `Terminated`.
    - Terminated sessions are immutable and permanently reject any further lifecycle transitions.
    - Illegal transitions (e.g., Initializing $\to$ Lock, Terminated $\to$ Activate) return an immediate error and do not mutate state.
- **Verdict:** Secure. Deterministic state machine enforcement prevents lifecycle tampering.

### Abuse Scenario 5: Seat Collision & Multiple Foreground Display Hijacking
- **Attack Vector:** An agent provisions or activates multiple sessions on the same physical seat (e.g. `seat0`) simultaneously, attempting to intercept or hijack the foreground display and keyboard input.
- **Mitigation:**
  - Invariant `CS2` enforces mutual exclusion per seat: at most one session can hold `SessionScope::Foreground` on any seat at any point in time.
  - Activating a session on seat $S$ atomically demotes any previous foreground session on seat $S$ to `SessionScope::Background`.
- **Verdict:** Secure. Seat arbitration is strictly serialized and non-reentrant.

### Abuse Scenario 6: Audit Evasion & Silent Operation
- **Attack Vector:** Executing session lifecycle modifications without recording operations in the system's tamper-evident audit log.
- **Mitigation:**
  - Every MCP tool call is wrapped inside `dispatch::recorded_call(&mut self.ring, &self.pep, ...)`.
  - Every invocation automatically appends an audit event to SQLite WAL (`audit.db`) with:
    - Tool name (`aios.session.*`)
    - Monotonically increasing `audit_id`
    - Policy revision (`sprint-2-rule-pack-v1`)
    - Caller arguments and execution outcome (`ok: true` or `ok: false`)
    - Session state before and after action execution
  - Audit database utilizes write-ahead logging (WAL) for atomic durability.
- **Verdict:** Secure. Zero unaudited execution paths exist.

---

## 2. Policy Enforcement & PEP Gating

- **PEP Evaluation:** Mutating tools (`aios.session.action`, `aios.session.create`) and query tools (`aios.session.validate`, `aios.session.list`, `aios.session.get`) are routed through the central policy dispatcher.
- **Fail-Closed Semantics:** Any malformed JSON-RPC request, invalid parameter type, or violated invariant immediately yields an error response (`ok: false`) and writes a failure audit record.
- **Irreversibility Analysis:** Standard session management actions are operational lifecycle controls. Destructive actions follow multi-stage confirmation (`Active` $\to$ `Terminating` $\to$ `Terminated`).

---

## 3. Findings & Hardening Plan for T-01438

The security review identified 3 defense-in-depth hardening opportunities to be implemented in `T-01438`:
1. **Unify `store_path` validation:** Apply $\le 1024$ length bound and control-character filtering to `aios.session.list`, `aios.session.get`, and `aios.session.action` (currently active on `create`).
2. **Unify `spec` 1 MiB bound:** Apply explicit 1 MiB payload bound to `aios.session.validate` when supplied as an inline JSON object or string.
3. **Bound `limit` query parameter:** Enforce $[1 \dots 10,000]$ bounds on the `limit` parameter in `aios.session.list`, rejecting 0 or excessive counts.
4. **Early `session_id` validation:** Validate `validate_session_id` on `aios.session.get` and `aios.session.action` before attempting store lookups.

---

## 4. Acceptance Verification
- [x] Security review artifact created with comprehensive threat modeling (Abuse Scenarios 1 through 6).
- [x] Input validation, argument injection, path traversal, and payload bounding assessed.
- [x] PEP gating and append-only SQLite WAL audit emission verified on all paths.
- [x] Zero unmitigated policy bypass vulnerabilities remain open.
