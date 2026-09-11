# T-01461: User Session Bootstrap — Security Policy: Research

## Metadata
- **Task ID:** `T-01461`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Security Policy (`code/aiosh-rust/aiosh-core::session_policy`)
- **Status:** Complete
- **Date:** 2026-09-11
- **Milestone:** Sub-Epic Launch: User Session Bootstrap (7/10) — Security Policy Research

---

## 1. Executive Summary & Objective

In the AIOS architecture, following core service supervision, user session data modeling, runtime service coordination, and configuration management, the **User Session Bootstrap Security Policy Subsystem** establishes deterministic policy enforcement, privilege containment, environment sanitization, seat isolation, and autonomous agent execution boundaries for interactive and AI-driven sessions.

The primary objective is to codify authoritative security invariants (`SSP1..SSP7`) governing:
1. **Identity & Privilege Containment**: Enforcing user/agent UID/GID restrictions, prohibiting untrusted root sessions, and isolating system greeter sessions.
2. **Session Type & Class Gating**: Validating session type compatibility (`Wayland`, `X11`, `Tty`, `AiAgent`, `WebRtc`) against designated classes (`User`, `Greeter`, `LockScreen`, `Agent`).
3. **Seat & Display Hardware Isolation**: Preventing remote sessions from hijacking physical console seats (`seat0`), sanitizing display strings, and bounding virtual terminal numbers.
4. **Environment Sanitization**: Disallowing dangerous dynamic linker and interpreter override variables (`LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`, `NODE_OPTIONS`, `PYTHONPATH`) and capping environment size.
5. **Quota & Concurrency Protection**: Restricting per-user active sessions and overall store capacity to prevent resource starvation.
6. **Agent Execution Sandboxing**: Gating autonomous AI agent sessions with capability boundaries and sandboxing policies.
7. **Policy Modes & Audit Trail**: Providing `Enforcing`, `Audit`, and `Permissive` modes with structured evaluation verdicts and immutable SQLite WAL audit logging.

---

## 2. Authoritative Sources & Upstream Standards

1. **Linux-PAM & Privilege Dropping**:
   - *Sources*: Linux-PAM System Administrators' Guide, `pam_env(8)`, `pam_limits(8)`, `pam_systemd(8)`.
   - Environment variables must be sanitized prior to session process spawning. Uncontrolled inheritance of `LD_PRELOAD` or `LD_LIBRARY_PATH` leads to local privilege escalation (LPE).
   - System accounts (UID < 1000) except greeters must be prevented from initiating unmanaged interactive graphical sessions.

2. **freedesktop.org systemd-logind & Console Seat Arbitration**:
   - *Sources*: `systemd-logind.service(8)`, `sd-login(3)`, `org.freedesktop.login1(5)`.
   - Multiseat security: `seat0` represents the primary local console with direct access to physical GPU, display, keyboard, and sound hardware.
   - Remote sessions (SSH, WebRTC, remote VNC) must not bind to `seat0` without explicit policy override to prevent physical console snooping.
   - Greeter class sessions (`SessionClass::Greeter`) must strictly run under dedicated display manager users (`lightdm`, `gdm`, `greeter`) and cannot transition into user scopes.

3. **POSIX & XDG Base Directory Isolation**:
   - *Sources*: POSIX.1-2017 `setsid(2)`, XDG Base Directory Specification v0.8.
   - Per-user runtime directory `/run/user/<UID>` must maintain `0700` permissions.
   - Environment key/value pairs must be bounded in length and count to prevent heap exhaustion or buffer overflows in downstream consumers.

4. **NIST SP 800-53 / CIS Linux Benchmarks**:
   - Identification and Authentication (IA-2), Least Privilege (AC-6), Concurrent Session Limits (AC-10).
   - Mandates strict caps on concurrent active user sessions to prevent resource exhaustion and denial of service.

---

## 3. Fact vs. Assumption Matrix

| Category | Authoritative Fact | Assumption / Engineering Decision |
|---|---|---|
| **Root Privileges** | Root (UID 0) interactive sessions bypass normal access controls and represent maximum blast radius. | By default, root interactive and agent sessions are disallowed (`disallow_root: true`) unless explicit allowlisting (`allowed_root_users: ["root"]`) is configured. |
| **Console Seat0** | `seat0` controls physical monitor and input devices. Remote access can spy on local console users. | Remote sessions (`remote_host.is_some()`) are disallowed from attaching to `seat0` unless `allow_remote_seat0` is explicitly set to true. |
| **Greeter Isolation** | Greeter sessions authenticate users before login and must never run as unprivileged regular users or general root. | Greeter sessions (`SessionClass::Greeter`) must run with dedicated system UIDs (< 1000 or specific user allowlist like `lightdm`). |
| **Environment Injection** | Variables like `LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS` can hijack library loading in executed binaries. | Security policy strictly disallows dynamic linker and language hook variables in session specifications by default. |
| **Agent Sessions** | Autonomous AI agents execute shell commands and tool calls; running them unrestricted risks host integrity. | Agent sessions (`SessionClass::Agent` or `SessionType::AiAgent`) require designated unprivileged UIDs and sandboxed execution contexts. |
| **Policy Modes** | Enterprise deployments require non-blocking trial periods before enforcement. | Three policy modes (`Enforcing`, `Audit`, `Permissive`) allow audit-only evaluation during deployment and fail-closed blocking in production. |

---

## 4. Invariant Catalog (`SSP1..SSP7`)

- **SSP1 (Identity & Root Boundaries)**:
  - `disallow_root` defaults to `true`. When active, sessions with `uid == 0` are rejected unless `username` is listed in `allowed_root_users`.
  - Greeter sessions (`SessionClass::Greeter`) must have `uid < 1000` or `username` matching `allowed_greeter_users`.
- **SSP2 (Session Type & Class Consistency)**:
  - `session_type` must be present in `allowed_session_types` (default: all 5 standard types).
  - Sessions of class `Greeter` must have a valid display (e.g. `:0`) and cannot specify a `remote_host`.
  - Sessions of class `Agent` must match `session_type == AiAgent`.
- **SSP3 (Seat & Hardware Protection)**:
  - If `remote_host.is_some()`, attaching to `seat0` is prohibited unless `allow_remote_seat0 == true`.
  - `vtnr` if specified must be in range $[1 \dots 64]$.
  - `display` if specified must conform to standard syntax (`:\d+` or `wayland-\d+`) $\le 32$ chars without control characters.
- **SSP4 (Environment Sanitization)**:
  - Environment variable names must not match any entry in `disallowed_env_vars` (`LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`, `NODE_OPTIONS`, `PYTHONPATH`, `RUBYOPT`, `PERL5OPT`).
  - Total environment variables count must not exceed `max_env_vars` (default: 256).
  - Variable keys $\le 256$ bytes, values $\le 4096$ bytes, no null bytes.
- **SSP5 (Concurrency & Quotas)**:
  - Active user session counts bounded by `max_sessions_per_user` (range $[1 \dots 128]$, default 32).
  - Total tracked sessions bounded by `max_total_sessions` (range $[10 \dots 10,000]$, default 1,024).
- **SSP6 (AI Agent Execution Constraints)**:
  - AI Agent sessions must run with `uid >= 1000` (non-root).
  - Unauthenticated agent sessions cannot be activated directly without authentication.
- **SSP7 (Policy Enforcement Modes & Audit Integrity)**:
  - Modes: `Enforcing` (fatal violations return `Err` / block action), `Audit` (violations recorded, action proceeds), `Permissive` (policy skipped with telemetry).
  - File loading bounded to $\le 64\text{ KiB}$ (`MAX_POLICY_FILE_BYTES`).
  - Every evaluation generates an immutable audit record in SQLite WAL.

---

## 5. Technical Decisions & Unknowns Resolved

1. **Standalone Policy Module vs Inline Invariants**:
   - *Decision*: Implement `code/aiosh-rust/aiosh-core/src/session_policy.rs`, paralleling `service_policy.rs` and `package_policy.rs`.
   - *Rationale*: Maintains uniform architectural pattern across AIOS Phase 1 subsystems, decouples policy specification from service runtime, and enables standalone policy loading from TOML/JSON configuration files.

2. **Verdict Structure**:
   - *Decision*: Define `SessionPolicyVerdict` returning `allowed: bool`, `mode: SessionPolicyMode`, and `violations: Vec<SessionPolicyViolation>` with explicit rule IDs (`SSP1..SSP7`).

3. **No External Dependencies**:
   - *Decision*: Implement entirely using existing workspace dependencies (`serde`, `serde_json`, `chrono`, `std::fs`).

---

## 6. Acceptance Confirmation
- [x] Authoritative sources consulted (Linux-PAM, freedesktop logind, POSIX, NIST).
- [x] Facts separated from assumptions.
- [x] Invariants `SSP1..SSP7` formally cataloged.
- [x] Zero source code changes in research phase.
