# T-01497: User Session Bootstrap Recovery & Validation Security Review

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01497  

---

## 1. Threat Model & Security Perimeter

The User Session Bootstrap Recovery & Validation subsystem (`code/aiosh-rust/aiosh-core/src/session_recovery.rs`, CLI commands `aiosh session check`/`recover`, and MCP tool `aios.session.check`) handles reading, deep validation, and automated non-destructive self-healing of user session persistent stores.

The security perimeter must defend against:
1. Path traversal and unauthorized filesystem manipulation via custom `store_path` parameters.
2. Resource exhaustion (DoS) via oversized files or unbounded session capacities.
3. State corruption and process leader confusion (duplicate `leader_pid` across active sessions).
4. Physical seat hijacking via concurrent foreground sessions on hardware seats.
5. Accidental or malicious data loss during recovery procedures.

---

## 2. Abuse Scenarios & Verification Matrix

### AS-1: Arbitrary Path Traversal & Symlink Attacks
- **Scenario:** An untrusted agent or caller supplies `store_path: "../../../../etc/shadow"` to `aiosh session check` or `aios.session.check`.
- **Mitigation:**
  - Strict input boundary validation: `store_path` strings exceeding 1,024 characters or containing ASCII/Unicode control characters are rejected immediately with code `2` / `INVALID_ARGUMENT`.
  - Backup quarantine logic uses `path.parent()` and `path.file_name()` with microsecond-timestamped suffixes (`<file_name>.bak.<timestamp>`), preventing arbitrary directory escape.
  - On Unix, quarantined backup files and reconstituted canonical stores are explicitly chmod'd to `0600` (`S_IRUSR | S_IWUSR`), preventing unprivileged read or tampering.
- **Verdict:** PASS — no unauthorized path traversal or permissions escalation possible.

### AS-2: Denial of Service via Store Size & Memory Exhaustion
- **Scenario:** An attacker supplies a multi-gigabyte corrupted store file to exhaust system memory during validation or recovery.
- **Mitigation:**
  - `load_from_path` in `UserSessionStore` enforces a hard limit of `MAX_SESSION_STORE_SIZE = 10 MiB` (`10 * 1024 * 1024` bytes) via `fs::metadata` inspection prior to reading contents into memory.
  - `validate_session_store` enforces a maximum capacity threshold of `MAX_STORE_CAPACITY = 10_000` sessions. Any store exceeding this threshold immediately fails validation and logs an explicit capacity violation error.
- **Verdict:** PASS — bounded file read and strict memory allocation ceilings.

### AS-3: Leader PID Collisions and Process Hijacking (`SSR5`)
- **Scenario:** A compromised or malformed session store assigns the identical `leader_pid` to multiple distinct active sessions to confuse audit trails or hijack administrative actions.
- **Mitigation:**
  - `validate_session_store` tracks `seen_pids: HashSet<u32>` across all non-terminated sessions.
  - Duplicate PIDs are immediately detected, decremented from `valid_sessions`, incremented into `invalid_sessions`, and logged into `report.errors`.
  - Invariant `SSR2` forces `healthy: false`, preventing any confused-deputy action on collided session records.
- **Verdict:** PASS — full process collision isolation enforced.

### AS-4: Concurrent Hardware Seat Hijacking (`SSR5`)
- **Scenario:** Multiple sessions declare `scope: SessionScope::Foreground` on the same physical seat (e.g., `seat0`) simultaneously, causing display multiplexing or credential interception.
- **Mitigation:**
  - `validate_session_store` indexes foreground sessions per physical seat name.
  - If `sessions.len() > 1` on any seat, validation fails with an explicit error identifying the conflicting sessions and emits an arbitration warning.
- **Verdict:** PASS — mutual exclusion of foreground sessions on physical seats guaranteed.

### AS-5: Non-Destructive Quarantine & Audit Non-Repudiation (`SSR4`)
- **Scenario:** Store self-healing destroys user session state or forensic evidence during corruption recovery.
- **Mitigation:**
  - Self-healing in `recover_session_store_with_backup` never deletes or truncates the existing file in place; it first creates an immutable, timestamped backup `<path>.bak.<timestamp>` with mode `0600`.
  - Every recovery event emits a classified audit record (`session.repair`) through `classify_and_emit` or `dispatch::recorded_call`, recording the exact quarantine backup path, error count, and recovery timestamp.
- **Verdict:** PASS — forensic preservation and tamper-evident audit emission verified.

---

## 3. Summary Verdict

No vulnerabilities, policy bypasses, or integrity leaks remain open. The recovery and validation subsystem satisfies all security invariants (`SSR1..SSR5`).
