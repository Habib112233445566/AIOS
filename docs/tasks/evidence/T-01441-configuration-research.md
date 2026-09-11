# T-01441: User Session Bootstrap — Configuration: Research

## Metadata
- **Task ID:** `T-01441`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** User Session Bootstrap Configuration Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Executive Summary

This research establishes the requirements, operational constraints, authoritative prior art, and configuration architecture for the AIOS User Session Bootstrap subsystem (`aiosh-core::session_config`).

In the preceding session bootstrap milestones (`T-01401..T-01440`), we implemented:
1. The formal Session Data Model and core validation invariants (`UserSessionSpec`, `UserSession`, `validate_session_spec`, `SB1..SB5`).
2. The core session manager, state transition engine (`Initializing` -> `Authenticating` -> `Active` <-> `Locked` -> `Terminating` -> `Terminated`), seat arbitration logic, and activity tracking (`UserSessionService`, `CS1..CS5`).
3. The operator CLI surface (`aiosh session` with subcommands `validate`, `list`, `show`, `action`, `create`, `activate`, `lock`, `unlock`, `terminate`, `auth`).
4. The autonomous agent Model Context Protocol (MCP) surface (`aios.session.*` across `validate`, `list`, `get`, `action`, `create` with persistent store binding).

Currently, operational parameters such as store paths (`.aios/session_store.json`), sizing limits (10 MiB ceiling, 1,024 sessions ceiling, 32 active sessions per user account), idle thresholds (default 900 seconds), and automatic lock timeouts are hardcoded or passed via ad-hoc CLI flags. A dedicated configuration subsystem (`SessionConfig`) is required to support centralized configuration loading, environment variable overrides, strict invariant validation, and deterministic runtime integration.

---

## 2. Existing Code & Architectural Patterns

Investigation of the configuration modules in `code/aiosh-rust/aiosh-core/src/` reveals established, robust design patterns that must be mirrored:

1. **`service_config.rs` (`ServiceConfig`)**:
   - Manages store paths, sizing limits, execution timeouts, and restart throttling.
   - Resolution hierarchy: Explicit configuration file -> Environment variables (`AIOS_SERVICE_*`) -> Safe built-in defaults.
   - Invariant validation enforcing bounds on file sizes (max 64 KiB config files, max 100 MiB store sizes, 10..100,000 entities).
2. **`package_config.rs` (`PackageConfig`)**:
   - Manages package store paths, sizing limits, default format types, and repository whitelist.
   - Resolution hierarchy: Explicit configuration file -> Environment variables (`AIOS_PACKAGE_*`) -> Safe built-in defaults.
3. **`session.rs` and `session_service.rs`**:
   - Hardcodes 10 MiB file size ceiling and 1,024 session entities ceiling on store loading.
   - Enforces max 32 concurrent active sessions per user account.
   - Uses temporary file rename (`tmp.<pid>.<nanos>`) for atomic disk writes.

---

## 3. Authoritative Sources & Citations

1. **`logind.conf(5)` (systemd-logind configuration)**:
   - Standardizes user session parameters: `KillUserProcesses=`, `IdleAction=`, `IdleActionSec=`, `InhibitDelayMaxSec=`, `UserStopDelaySec=`, and per-user session limits (`SessionsMax=`).
   - Citation: systemd v255 manual pages, *logind.conf(5)*.
2. **PAM (`pam.conf(5)`, `pam_limits.conf(5)`)**:
   - Defines system resource boundaries per user session (`maxlogins`, `priority`, CPU, memory, open files).
   - Citation: Linux-PAM documentation, *pam_limits(8)*.
3. **freedesktop.org XDG Base Directory Specification**:
   - Governs standard location separation between declarative configuration (`$XDG_CONFIG_HOME` / `config/` / `/etc/aios/`) and dynamic state stores (`$XDG_DATA_HOME` / `.aios/` / `/var/lib/aios/`).
   - Citation: freedesktop.org XDG Base Directory Specification v0.8.
4. **ADR-0035 (AIOS Audit & Capability Governance)**:
   - Mandates deterministic configuration resolution, explicit error handling without silent fallbacks on corrupted settings, and non-repudiable audit recording for any administrative configuration mutations.

---

## 4. Facts vs. Assumptions

| Item | Status | Details |
|---|---|---|
| In-memory store limits | **Fact** | `UserSessionService` currently hardcodes a 10 MiB byte limit and 1,024 session count limit. |
| Max sessions per user | **Fact** | `UserSessionService` currently bounds active sessions per user to 32. |
| CLI `--store` flag | **Fact** | `aiosh session` commands accept `--store <path>`, falling back to `.aios/session_store.json` if omitted. |
| MCP tool parameter | **Fact** | All session MCP tools accept an optional `store_path` parameter bounded to 1,024 characters, resolving to `.aios/session_store.json`. |
| Default idle timeout | **Fact** | `UserSessionSpec` defaults to `idle_timeout_seconds = 900` (15 minutes). |
| Config resolution precedence | **Assumption (to codify)** | File configuration takes precedence over environment variables, which take precedence over built-in defaults: `Config File > Environment Variables > Safe Defaults`. |
| Default config file location | **Assumption (to codify)** | Declarative configuration should default to `config/session.json` (or `.aios/session_config.json`). |
| Default runtime store path | **Assumption (to codify)** | Dynamic session store should default to `.aios/session_store.json`. |
| Auto-persistence behavior | **Assumption (to codify)** | `auto_persist = true` by default so mutations executed via CLI or MCP persist across multi-turn invocations. |

---

## 5. Unknowns & Decisions Needed

### Decision 1: Default Store Path and Configuration Path
- **State Store Path:** `.aios/session_store.json` (preserves multi-turn state across CLI and MCP sessions).
- **Configuration Path:** `config/session.json` (checked first; optional, with safe in-memory defaults if not present).
- **Recommendation:** Adopt `.aios/session_store.json` for runtime data and `config/session.json` for configuration settings.

### Decision 2: Environment Variable Prefix
- **Option A:** `AIOS_SESSION_*` (e.g. `AIOS_SESSION_STORE_PATH`, `AIOS_SESSION_MAX_PER_USER`, `AIOS_SESSION_MAX_TOTAL`, `AIOS_SESSION_IDLE_TIMEOUT_SECS`, `AIOS_SESSION_MAX_STORE_SIZE_BYTES`, `AIOS_SESSION_AUTO_PERSIST`).
- **Option B:** `AIOSH_SESSION_*`.
- **Recommendation:** Option A (`AIOS_SESSION_*`), matching `AIOS_SERVICE_*`, `AIOS_PACKAGE_*`, `AIOS_IMAGE_*`, and `AIOS_DISTRO_*`.

### Decision 3: Configuration Invariants (`SC1..SC7`)
1. `SC1`: Store path validity: non-empty, $\le 1024$ bytes, no ASCII control characters or null bytes.
2. `SC2`: Max sessions per user account: bounded to $[1 \dots 128]$ (default: 32).
3. `SC3`: Max total sessions in store: bounded to $[10 \dots 10,000]$ (default: 1,024).
4. `SC4`: Default idle timeout seconds: bounded to $[10 \dots 86,400]$ seconds (default: 900s).
5. `SC5`: Max store size bytes: bounded to $[64\text{ KiB} \dots 100\text{ MiB}]$ (default: 10 MiB).
6. `SC6`: Precedence and deterministic resolution: Config file overrides env vars; env vars override built-in defaults.
7. `SC7`: Safe parsing bounds: Config file on disk must not exceed 64 KiB; invalid JSON or invalid values fail loudly with explicit errors.
