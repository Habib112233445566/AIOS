# T-01471: User Session Bootstrap — Observability: Research

## Metadata
- **Task ID:** `T-01471`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Observability Subsystem (`code/aiosh-rust/aiosh-core::session_observability`)
- **Status:** Complete
- **Date:** 2026-09-11
- **Milestone:** Sub-Epic Launch: User Session Bootstrap (8/10) — Observability Research

---

## 1. Executive Summary & Objective

In modern multi-user, multi-seat OS architectures and AI execution substrates, operational observability of user and agent sessions is vital for security monitoring, resource accounting, idle timeout enforcement, and automated triage.

The User Session Bootstrap Observability subsystem must aggregate real-time session inventory, state distributions, focus scopes, idle metrics, and security policy compliance into structured telemetry reports (`SessionObservabilityReport`) consumable by operators and autonomous agents over CLI and MCP.

---

## 2. Authoritative Sources & Upstream Standards

1. **freedesktop.org loginctl & systemd-logind D-Bus API**:
   - *Sources*: `loginctl(1)`, `org.freedesktop.login1(5)`, `sd-login(3)`.
   - Introspection primitives: `ListSessions`, `ListUsers`, `ListSeats`, `Session.IdleHint`, `Session.IdleSinceHint`, `Session.State`, `Session.Class`, `Session.Type`, `Session.Active`.
   - Telemetry model: Distinguishes physical seat assignment (`seat0`, `seat1`), session states (`online`, `active`, `closing`), and user identity binding.

2. **POSIX User Accounting (`utmp(5)`, `wtmp(5)`)**:
   - *Sources*: POSIX.1-2017 `getutxent(3)`, `who(1)`, `w(1)`, `last(1)`.
   - Tracks login time, terminal name (`tty1`), hostname (for remote sessions), and idle duration based on TTY character device mtime.

3. **OpenTelemetry Metrics Semantic Conventions for User Sessions**:
   - Gauges for active concurrent sessions (`system.sessions.active`), distribution histograms for session lifespan and idle duration (`system.sessions.idle_seconds`), and counter for lifecycle state changes.

4. **NIST SP 800-53 AC-10 / AU-6 (Audit Review & Telemetry)**:
   - Requires continuous monitoring and periodic reporting of concurrent session capacity, inactivity timeouts, and security policy violations.

---

## 3. Fact vs. Assumption Matrix

| Category | Authoritative Fact | Assumption / Engineering Decision |
|---|---|---|
| **Session States** | Sessions transition between Initializing, Authenticating, Active, Locked, Terminating, Terminated. | Observability reports must aggregate a full state histogram (`state_breakdown`) rather than simple boolean active/inactive flags. |
| **Seat Focus** | Only one session per physical seat can hold foreground input focus simultaneously. | Reports must break down foreground vs background scopes per seat (`seat_breakdown` and `scope_breakdown`). |
| **Idle Duration** | Inactivity triggers lock screens and terminal timeouts. | Reports calculate active idle counts, max idle seconds, and mean idle duration across active sessions. |
| **Multi-Tenancy** | Individual users have concurrency quotas ($\le 32$). | Reports must aggregate session counts per user (`user_breakdown`) and compute distinct user count. |
| **Policy Compliance** | Security violations must be visible to security operations without interrupting audit logs. | Reports incorporate an optional `UserSessionSecurityPolicy` evaluation, reporting compliant counts, violation counts, and specific rule IDs triggered. |

---

## 4. Invariant Catalog (`SSO1..SSO6`)

- **SSO1 (Inventory & State Distribution)**:
  - Aggregates `total_sessions`, `distinct_users_count`, and a complete `state_breakdown` (`Initializing`, `Authenticating`, `Active`, `Locked`, `Terminating`, `Terminated`).
- **SSO2 (Seat & Scope Telemetry)**:
  - Generates `seat_breakdown` (counts per physical seat name) and `scope_breakdown` (`Foreground`, `Background`).
- **SSO3 (Class & Type Distribution)**:
  - Aggregates `session_type_breakdown` (`Wayland`, `X11`, `Tty`, `AiAgent`) and `session_class_breakdown` (`User`, `Greeter`, `LockScreen`, `Agent`).
- **SSO4 (Idle & Lock Activity Metrics)**:
  - Reports `locked_count`, `idle_sessions_count` (idle > 0), `max_idle_seconds`, and `total_idle_seconds`.
- **SSO5 (User Concurrency Distribution)**:
  - Maps `user_breakdown` (session count per username), identifying users exceeding threshold quotas.
- **SSO6 (Policy Compliance Telemetry)**:
  - If a `UserSessionSecurityPolicy` is provided, reports `policy_compliant_count`, `policy_violations_count`, and a list of violating session IDs with violation summaries.

---

## 5. Technical Decisions & Scope

1. **Dedicated Module**: Implement `code/aiosh-rust/aiosh-core/src/session_observability.rs` exposing `SessionObservabilityReport`.
2. **Integration Test Suite**: Implement `code/aiosh-rust/aiosh-core/tests/test_session_observability.rs`.
3. **CLI & MCP Surface**:
   - Add `aiosh session stats [--policy <path>] [--store <path>] [--json]` to CLI.
   - Add `aios.session.stats` tool to MCP server.
4. **Zero New Dependencies**: Reuses standard workspace serialization and datetime crates.

---

## 6. Acceptance Confirmation
- [x] Authoritative sources consulted (systemd-logind, POSIX utmp, OpenTelemetry, NIST).
- [x] Facts separated from assumptions.
- [x] Invariants `SSO1..SSO6` formally cataloged.
- [x] Zero source code changes in research phase.
