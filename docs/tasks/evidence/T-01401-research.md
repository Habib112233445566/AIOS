# T-01401: User Session Bootstrap - Data Model: Research

## Metadata
- **Task ID:** `T-01401`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic Launch: User Session Bootstrap (1/10) — Data Model Research

---

## 1. Executive Overview & Mission Context

In the AIOS architecture, following the initialization of PID 1 and core daemon supervision (`T-01301..T-01400`), the **User Session Bootstrap** subsystem is responsible for orchestrating, authenticating, configuring, and tracking interactive user environments and autonomous AI execution contexts.

The operational vision for AIOS unites three pillars:
1. **Pillar A (Ethical Hacking on the Inside)**: Underlying Kali Linux Rolling distribution with access to penetration testing tools and kernel capabilities.
2. **Pillar B (Windows-Look Desktop on the Outside)**: Highly accessible graphical desktop environment (XFCE with `kali-undercover` or KDE Plasma Fluent theme) delivered via display managers and seat controllers.
3. **Pillar C (S-Rank AI Kernel Subsystem)**: An autonomous AI shell (`ai_agent.py` / local SLM) operating as a first-class desktop companion and headless agent with policy-gated access (PEP) and SQLite WAL non-repudiation auditing.

The User Session Bootstrap data model must canonically represent and govern:
- Interactive TTY console sessions (agetty / login).
- Graphical display sessions (X11 / Wayland via LightDM, Greetd, or SDDM).
- Autonomous AI companion sessions (headless execution contexts, desktop co-pilot sessions).
- PAM authentication workflows and credential binding.
- Seat allocation (`seat0`, virtual terminals `tty1..tty7`, display identifiers `:0`, `:1`).
- XDG Base Directory specification variables (`XDG_RUNTIME_DIR`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_SESSION_TYPE`, `XDG_SESSION_CLASS`).
- Session isolation, cgroup slicing (`user-<UID>.slice` / `session-<ID>.scope`), and security policy controls.

---

## 2. Authoritative Sources & Upstream Standards

1. **Linux-PAM (Pluggable Authentication Modules) Architecture**:
   - *Sources*: Linux-PAM System Administrators' Guide, RFC 86.0 (X/Open Single Sign-On PAM).
   - Core API primitives: `pam_start(3)`, `pam_authenticate(3)`, `pam_acct_mgmt(3)`, `pam_open_session(3)`, `pam_close_session(3)`, `pam_end(3)`.
   - Configuration hierarchy in `/etc/pam.d/`: `common-auth`, `common-account`, `common-session`, `common-password`, `lightdm`, `login`, `su`, `sudo`.
   - Session establishment: `pam_setcred(3)` sets up process credentials (UID, GID, supplementary groups); `pam_open_session(3)` initializes session modules (`pam_systemd.so`, `pam_limits.so`, `pam_env.so`, `pam_umask.so`).

2. **freedesktop.org systemd-logind & sd-login(3) Specification**:
   - *Sources*: `systemd-logind.service(8)`, `sd-login(3)`, `org.freedesktop.login1(5)` D-Bus interface.
   - Session identifiers: Unique strings (e.g. `"c1"`, `"2"`, `"sess-01"`).
   - Session classes: `user` (normal user session), `greeter` (login screen / display manager), `lock-screen` (screensaver / locker), `background` (cron, lingering user services, headless daemons).
   - Session types: `unspecified`, `tty`, `x11`, `wayland`, `mir`, `web`.
   - Seat management: Multiseat architecture anchored at `seat0` (primary display, keyboard, mouse), with dynamic seat assignment for secondary hardware.
   - Session states: `online` (session processes running), `active` (session currently in foreground on seat), `closing` (terminating processes).

3. **freedesktop.org XDG Base Directory Specification (v0.8+)**:
   - *Sources*: XDG Base Directory Specification, freedesktop.org.
   - `XDG_RUNTIME_DIR`: Per-user volatile directory on `tmpfs` (e.g., `/run/user/<UID>`), permissions `0700`, owned by user. Must be removed when last session terminates unless user lingering is enabled.
   - Standard variables: `XDG_CONFIG_HOME` (`~/.config`), `XDG_DATA_HOME` (`~/.local/share`), `XDG_CACHE_HOME` (`~/.cache`), `XDG_STATE_HOME` (`~/.local/state`).
   - Session context variables: `XDG_SESSION_ID`, `XDG_SESSION_TYPE`, `XDG_SESSION_CLASS`, `XDG_SEAT`, `XDG_VTNR`.

4. **POSIX.1-2017 Process Group & Session Semantics**:
   - *Sources*: IEEE Std 1003.1-2017, Section 3.344 (Session), `setsid(2)`, `getsid(2)`, `tcsetpgrp(3)`.
   - A session is a collection of one or more process groups.
   - The process creating the session via `setsid()` becomes the session leader (PID == SID) and process group leader.
   - A session may have at most one controlling terminal (TTY), which routes terminal-generated signals (`SIGINT`, `SIGQUIT`, `SIGHUP`) to the foreground process group.

5. **Display Managers & Desktop Shell Bootstrapping (Kali Linux / Debian)**:
   - *Sources*: `lightdm(1)`, `greetd(1)`, `xinit(1)`, Kali Undercover specification.
   - Kali Linux default display manager: `lightdm` with `lightdm-gtk-greeter` or `greetd`.
   - Graphical session startup: User authenticates via PAM -> Greeter invokes session wrapper (`/etc/X11/Xsession` or `/usr/bin/startxfce4`) -> launches window manager / desktop components (`xfwm4`, `xfce4-panel`, `xfdesktop`).
   - `kali-undercover`: Shell script and XFCE setting toggle switching panel, theme, window borders, and icons between dark Kali and Windows 10 style.

6. **NIST SP 800-53 (AC-2 Account Management, AC-11 Session Lock, AC-12 Session Termination)**:
   - AC-2: Explicit user identity binding, attribute validation.
   - AC-11: Session locking capabilities preventing unauthorized access after inactivity or operator trigger.
   - AC-12: Orderly session termination with resource cleanup and audit record generation.

---

## 3. Facts vs. Assumptions

| Fact | Assumption |
|---|---|
| Linux user sessions rely on PAM for authentication, credential allocation, and environment setup. | AIOS can provide a unified, memory-safe Rust data model representing both human interactive sessions and autonomous AI agent sessions. |
| In production, `systemd-logind` (or `elogind`/standalone session daemon) creates `/run/user/<UID>` and manages seats. | In containerized, headless, or lightweight test environments, AIOS can synthesize or mock session lifecycle states deterministically without requiring root D-Bus daemons. |
| Graphical sessions require an active seat, display designation (`:0`), and XDG environment variables. | AIOS can represent TTY, X11, Wayland, and AI Agent sessions under a shared schema with common lifecycle operations (`create`, `authenticate`, `activate`, `lock`, `unlock`, `terminate`). |
| Uncontrolled session creation can exhaust system memory, file descriptors, and process slots (PID exhaustion). | Enforcing formal capacity limits (`SB1..SB5`: $\le 32$ sessions per user, $\le 1,024$ total sessions) will guarantee bounded memory and deterministic behavior. |
| All security-sensitive state transitions (login, session lock, session termination) must be auditable under ADR-0035/ADR-0036. | Session queries (`aiosh session list`, `aios.session.get`) are non-state-changing queries and do not require emitting audit records. |

---

## 4. Proposed Data Model & Invariants

### 4.1 Core Types (`code/aiosh-rust/aiosh-core/src/session.rs`)

1. **`SessionType`**: Enum
   - `Tty`: Text console session bound to a virtual terminal (e.g. `/dev/tty1`).
   - `X11`: X Window System graphical session (e.g. XFCE / Kali Undercover on `:0`).
   - `Wayland`: Wayland compositor graphical session.
   - `AiAgent`: Autonomous AI agent execution session (headless or co-pilot desktop).

2. **`SessionClass`**: Enum
   - `User`: Standard authenticated human operator session.
   - `Greeter`: Display manager / login prompt session prior to user authentication.
   - `LockScreen`: Screen lock overlay session.
   - `Background`: Autonomous background or non-interactive service session.
   - `Agent`: Autonomous AI assistant companion session.

3. **`SessionState`**: Enum
   - `Initializing`: Session record created, pending credential setup.
   - `Authenticating`: PAM or key exchange in progress.
   - `Active`: User logged in, session running in foreground or background.
   - `Locked`: Session temporarily locked; requires re-authentication.
   - `Terminating`: Tear-down signals dispatched (`SIGTERM`), awaiting process exit.
   - `Terminated`: Session completely closed; resources released.

4. **`SessionScope`**: Enum
   - `Foreground`: Directly receiving seat input / screen focus.
   - `Background`: Running detached or switched to another VT.

5. **`UserSessionSpec`**: Specification for creating or configuring a session:
   - `session_id: String` (validated identifier, e.g. `sess-01`, `c1`)
   - `username: String` (validated POSIX username, e.g. `kali`, `root`, `aios-agent`)
   - `uid: u32` (POSIX user ID)
   - `gid: u32` (POSIX primary group ID)
   - `session_type: SessionType`
   - `session_class: SessionClass`
   - `seat: String` (seat identifier, e.g. `seat0`)
   - `vtnr: Option<u32>` (virtual terminal number, e.g. `1..12`)
   - `display: Option<String>` (e.g. `":0"`, `":1"`)
   - `remote_host: Option<String>` (IP or hostname for SSH / remote access)
   - `environment: BTreeMap<String, String>` (bounded environment map)

6. **`UserSessionStatus`**: Runtime status snapshot of an active session:
   - `session_id: String`
   - `username: String`
   - `uid: u32`
   - `state: SessionState`
   - `scope: SessionScope`
   - `leader_pid: Option<u32>` (session leader process ID)
   - `created_at: String` (ISO 8601 timestamp)
   - `last_active_at: String` (ISO 8601 timestamp)
   - `idle_seconds: u64`
   - `locked: bool`

7. **`UserSessionAction`**: Lifecycle transition requests:
   - `Create`, `Authenticate`, `Activate`, `Lock`, `Unlock`, `Terminate`

8. **`UserSessionQuery`**: Query criteria for listing/filtering sessions:
   - `username: Option<String>`
   - `state: Option<SessionState>`
   - `session_type: Option<SessionType>`
   - `seat: Option<String>`
   - `limit: Option<usize>`

9. **`UserSessionStore`**: Canonical in-memory & file-backed store for all active and recently terminated sessions:
   - `sessions: BTreeMap<String, UserSessionStatus>`
   - `specs: BTreeMap<String, UserSessionSpec>`
   - `version: u32`

---

### 4.2 Formal Subsystem Invariants (`SB1..SB5`)

- **`SB1` (Session Identifier Syntax & Bounding)**:
  - Session IDs must match `^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,63}$`.
  - Length must be between 1 and 64 characters.
  - No path traversal (`..`), whitespace, or shell metacharacters permitted.

- **`SB2` (Username & Identity Bounds)**:
  - Usernames must match POSIX standard `^[a-z_][a-z0-9_-]{0,31}$` (or root / AIOS agent conventions).
  - Length between 1 and 32 characters.
  - UID and GID must be valid unsigned integers ($\le 2,147,483,647$).

- **`SB3` (State Machine & Transition Validity)**:
  - Valid forward lifecycle transitions only:
    - `Initializing` $\to$ `Authenticating` | `Terminating`
    - `Authenticating` $\to$ `Active` | `Terminated`
    - `Active` $\to$ `Locked` | `Terminating`
    - `Locked` $\to$ `Active` (via Unlock) | `Terminating`
    - `Terminating` $\to$ `Terminated`
  - Any attempt to execute an invalid transition (e.g. `Terminated` $\to$ `Active` or `Locked` $\to$ `Authenticating`) must return an explicit `Err`.

- **`SB4` (Environment & Path Isolation)**:
  - Environment variable keys must match `^[A-Z_][A-Z0-9_]{0,63}$`.
  - Values bounded to 4,096 characters per variable.
  - Total environment keys $\le 256$ per session.
  - `XDG_RUNTIME_DIR` if specified must be an absolute path (`/run/user/<UID>`).

- **`SB5` (Resource Caps & Sizing Boundaries)**:
  - Maximum concurrent sessions per user: $\le 32$.
  - Maximum system-wide sessions in store: $\le 1,024$.
  - Store serialization file size ceiling: $\le 10$ MiB.

---

## 5. Integration Architecture with AIOS Subsystems

```mermaid
graph TD
    subgraph "Phase 1 - Linux Base System"
        INIT["PID 1 Init (systemd / OpenRC)<br><i>T-01301..T-01400</i>"] --> USB["User Session Bootstrap<br><i>T-01401..T-01500</i>"]
        USB --> PAM["Linux-PAM<br>/etc/pam.d/"]
        USB --> SEAT["Seat Controller<br>seat0, VT1..VT7"]
        USB --> XDG["XDG Base Environment<br>/run/user/UID"]
    end

    subgraph "Session Runtimes"
        USB --> S_X11["X11 Desktop Session<br>XFCE / Kali Undercover (:0)"]
        USB --> S_TTY["Virtual Console<br>agetty / login (tty1)"]
        USB --> S_AI["AI Agent Session<br>Headless & Co-pilot"]
    end

    subgraph "AIOS Governance & Control"
        CLI["aiosh session ...<br>Operator CLI Surface"] --> USB
        MCP["aios.session.*<br>Agent MCP API Surface"] --> USB
        PEP["Policy Enforcement Point (PEP)<br>Capability Validation"] --> USB
        AUDIT["Audit Ring (audit.rs)<br>SQLite WAL Non-Repudiation"] <-- USB
    end
```

---

## 6. Unknowns & Decisions Needed

Before proceeding to specification (`T-01402`) and scaffold (`T-01403`):

1. **Decision on Standalone Mock vs Native D-Bus Binding**:
   - *Question*: Should `aiosh-core::session` directly depend on `zbus` / D-Bus to talk to `org.freedesktop.login1`, or use a standalone, canonical Rust state store with an abstracted adapter pattern?
   - *Decision*: **Standalone canonical store with backend abstraction** (consistent with `package.rs` and `service.rs`). This allows 100% testability on non-Linux build hosts, deterministic regression testing, and zero runtime crashes if D-Bus is unavailable in containerized or chroot environments.

2. **Decision on AI Agent Session Distinction**:
   - *Question*: Should AI agent sessions be modeled as standard `user` sessions, or have a first-class `SessionType::AiAgent` / `SessionClass::Agent`?
   - *Decision*: **First-class `SessionType::AiAgent` and `SessionClass::Agent`**. AI agent sessions have unique attributes: headless execution, capability grant tokens, autonomous supervision, and distinct audit requirements under ADR-0035.

3. **Decision on Persistence Format**:
   - *Question*: How should active session records be persisted?
   - *Decision*: **Atomic JSON store** at `/run/aios/sessions.json` with fallback to in-memory state. `/run` is volatile tmpfs, ensuring stale session state is cleanly wiped across system reboots.

4. **Decision on Virtual Terminal (VT) Allocation**:
   - *Question*: What VT range should AIOS reserve for graphical vs console vs agent sessions?
   - *Decision*: `tty1..tty6` for text console login, `tty7` for primary graphical desktop (X11 / LightDM), and virtual headless seats for AI agent sessions.

---

## 7. Acceptance Verification
- [x] Authoritative sources collected and cited (Linux-PAM, freedesktop logind, XDG Base Directory, POSIX.1-2017, Kali Linux Undercover, NIST SP 800-53).
- [x] Facts vs. assumptions explicitly tabulated.
- [x] Proposed data model, enums, structs, and invariant equations (`SB1..SB5`) defined.
- [x] Unknowns and architectural decisions explicitly resolved.
- [x] No code files under `code/` modified during research phase.
