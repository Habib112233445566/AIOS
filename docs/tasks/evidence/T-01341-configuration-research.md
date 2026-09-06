# T-01341: Init & Service Supervision - Configuration: Research

## Metadata
- **Task ID:** `T-01341`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Init & Service Supervision Configuration Subsystem
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Executive Summary

This research establishes the requirements, operational constraints, authoritative prior art, and configuration architecture for the AIOS Init & Service Supervision subsystem (`aiosh-core::service_config`).

In the preceding milestones (`T-01301..T-01340`), we implemented:
1. The formal Service Data Model and core validation invariants (`ServiceSpec`, `ServiceStatus`, `validate_service_spec`, `SS1..SS5`).
2. The in-memory registry, topological cycle-detecting dependency planner (Kahn's algorithm), and lifecycle state machine transitions (`ServiceStore`, `CS1..CS5`).
3. The operator CLI surface (`aiosh service` with subcommands `validate`, `list`, `show`, `action`, `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`, `order`).
4. The autonomous agent Model Context Protocol (MCP) surface (`aios.service.*` across all 5 standard JSON-RPC tools with persistent store binding).

Currently, operational parameters such as store paths, sizing limits (10 MiB ceiling, 10,000 entities ceiling), timeout bounds (30 seconds), and restart throttling parameters are hardcoded or passed via ad-hoc CLI flags. A dedicated configuration subsystem (`ServiceConfig`) is required to support centralized configuration loading, environment variable overrides, strict invariant validation, and deterministic runtime integration.

---

## 2. Existing Code & Architectural Patterns

Investigation of the configuration modules in `code/aiosh-rust/aiosh-core/src/` reveals established, robust design patterns that must be mirrored:

1. **`package_config.rs` (`PackageConfig`)**:
   - Manages package store paths, sizing limits, default format types (`deb`/`apk`), and repository whitelist.
   - Resolution hierarchy: Explicit configuration file -> Environment variables (`AIOS_PACKAGE_*`) -> Safe built-in defaults.
   - Invariant validation enforcing bounds on file sizes (max 64 KiB config files, max 100 MiB store sizes, 10..100,000 entities).
2. **`distro_config.rs` (`DistroConfig`)**:
   - Manages store paths, reference distribution profiles, and evaluation weights.
   - Resolution hierarchy: Explicit file -> Environment variables (`AIOS_DISTRO_*`) -> Built-in defaults.
3. **`base_image_config.rs` (`ImageBuildConfig`)**:
   - Manages build and output artifact directories, execution timeouts, max image sizing, and compression levels.
   - Resolution hierarchy: `--config` file -> Environment variables (`AIOS_IMAGE_*`) -> Embedded defaults.
   - Invariants `CF1..CF6` verified with unit and integration test suites.
4. **`ledger_config.rs` (`LedgerConfig`)**:
   - Manages lock timeouts and file paths with environment overrides (`AIOS_LEDGER_*`).
5. **`service_service.rs` (`ServiceStore`)**:
   - Hardcodes 10 MiB file size ceiling and 10,000 service entities ceiling on `load_from_path`.
   - Uses temporary file rename (`tmp.<pid>`) for atomic disk writes.

---

## 3. Authoritative Sources & Citations

1. **`systemd.service(5)` & `systemd.unit(5)`**:
   - Defines system configuration hierarchies: vendor units (`/lib/systemd/system/`), volatile runtime units (`/run/systemd/system/`), and administrator overrides (`/etc/systemd/system/`).
   - Standardizes service timeout configurations (`TimeoutStartSec=`, `TimeoutStopSec=`, defaulting to 90s) and restart rate limiting (`StartLimitIntervalSec=`, `StartLimitBurst=`).
   - Citation: systemd v255 manual pages, *systemd.service(5)*, *systemd.unit(5)*.
2. **freedesktop.org XDG Base Directory Specification**:
   - Governs standard location separation between declarative configuration (`$XDG_CONFIG_HOME` / `config/` / `/etc/aios/`) and dynamic state stores (`$XDG_DATA_HOME` / `.aios/` / `/var/lib/aios/`).
   - Citation: freedesktop.org XDG Base Directory Specification v0.8.
3. **S6 & Runit Supervision Architectures**:
   - Documents deterministic, minimal supervision tree configurations: per-service timeout definitions, supervised run states, and static dependency declaration without dynamic race conditions.
   - Citation: Laurent Bercot, *S6: Supervision and Init Systems Architecture*, Skarnet.org.
4. **ADR-0035 (AIOS Audit & Capability Governance)**:
   - Mandates deterministic configuration resolution, explicit error handling without silent fallbacks on corrupted settings, and non-repudiable audit recording for any administrative configuration mutations.

---

## 4. Facts vs. Assumptions

| Item | Status | Details |
|---|---|---|
| In-memory store limits | **Fact** | `ServiceStore::load_from_path` currently hardcodes a 10 MiB byte limit and 10,000 service count limit. |
| CLI `--store` flag | **Fact** | `aiosh service` commands accept `--store <path>`, falling back to `.aios/service_store.json` if omitted. |
| MCP tool parameter | **Fact** | All service MCP tools accept an optional `store_path` parameter bounded to 1,024 characters, resolving to `.aios/service_store.json`. |
| Default service timeouts | **Fact** | `ServiceSpec` initializes canonical services with `timeout_start_secs = 30` and `timeout_stop_secs = 30`. |
| Config resolution precedence | **Assumption (to codify)** | File configuration takes precedence over environment variables, which take precedence over built-in defaults: `Config File > Environment Variables > Safe Defaults`. |
| Default config file location | **Assumption (to codify)** | Declarative configuration should default to `config/service.json` (or `.aios/service_config.json`). |
| Default runtime store path | **Assumption (to codify)** | Dynamic service store should default to `.aios/service_store.json` (consistent with `.aios/packages.json` and `.aios/triage_store.json`). |
| Auto-persistence behavior | **Assumption (to codify)** | `auto_persist = true` by default so mutations executed via CLI or MCP persist across multi-turn invocations. |

---

## 5. Unknowns & Decisions Needed

### Decision 1: Default Store Path and Configuration Path
- **State Store Path:** `.aios/service_store.json` (preserves multi-turn state across CLI and MCP sessions).
- **Configuration Path:** `config/service.json` (checked first; optional, with safe in-memory defaults if not present).
- **Recommendation:** Adopt `.aios/service_store.json` for runtime data and `config/service.json` for configuration settings.

### Decision 2: Environment Variable Prefix
- **Option A:** `AIOS_SERVICE_*` (e.g. `AIOS_SERVICE_STORE_PATH`, `AIOS_SERVICE_DEFAULT_TIMEOUT_START_SECS`, `AIOS_SERVICE_DEFAULT_TIMEOUT_STOP_SECS`, `AIOS_SERVICE_MAX_STORE_SIZE_BYTES`, `AIOS_SERVICE_MAX_ENTITIES`, `AIOS_SERVICE_AUTO_PERSIST`).
- **Option B:** `AIOSH_SERVICE_*`.
- **Recommendation:** Option A (`AIOS_SERVICE_*`), matching `AIOS_PACKAGE_*`, `AIOS_IMAGE_*`, and `AIOS_DISTRO_*`.

### Decision 3: Configuration Schema Fields (`ServiceConfig`)
The configuration struct `ServiceConfig` should encompass:
1. `store_path: PathBuf` (Default: `.aios/service_store.json`).
2. `default_timeout_start_secs: u32` (Default: `30`; valid range: `1 .. 3600`).
3. `default_timeout_stop_secs: u32` (Default: `30`; valid range: `1 .. 3600`).
4. `max_store_size_bytes: u64` (Default: `10 * 1024 * 1024` = 10 MiB; valid range: `64 KiB .. 100 MiB`).
5. `max_entity_count: usize` (Default: `10,000`; valid range: `10 .. 100,000`).
6. `auto_persist: bool` (Default: `true`).
7. `restart_backoff_secs: u32` (Default: `5`; valid range: `1 .. 300`).
8. `max_restart_burst: u32` (Default: `5`; valid range: `1 .. 50`).

---

## 6. Acceptance Criteria Verification
- [x] Authoritative sources collected and cited (`systemd.service(5)`, `systemd.unit(5)`, XDG Base Directory Specification, S6, ADR-0035).
- [x] Facts separated from assumptions.
- [x] No production code modified during research phase.
- [x] Explicit decisions listed for the specification task (`T-01342`).
