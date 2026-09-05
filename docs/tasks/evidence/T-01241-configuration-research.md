# T-01241: Package Management - Configuration: Research

## Metadata
- **Task ID:** `T-01241`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Package Management Configuration Subsystem
- **Status:** Complete

## 1. Executive Summary
This research establishes the requirements, constraints, prior art, and configuration architecture for the AIOS Package Management subsystem (`aiosh-core::package_config`).
In the preceding milestones (`T-01201..T-01240`), we implemented the unified package data model (`PackageSpec`), the in-memory core service registry (`PackageStore`), the operator CLI surface (`aiosh package`), and the autonomous agent MCP surface (`aios.package.*`).
Currently, operational parameters such as store paths, sizing limits (10 MiB / 10,000 entities), and default format selections are hardcoded or passed via ad-hoc CLI flags. A dedicated configuration subsystem is required to support centralized configuration loading, environment overrides, and strict invariant validation.

---

## 2. Existing Code & Architectural Patterns
Investigation of the existing configuration modules in `code/aiosh-rust/aiosh-core/src/` reveals consistent design patterns:
1. **`distro_config.rs` (`DistroConfig`)**:
   - Manages store paths, pinned reference profile IDs, scoring weights, and evaluation thresholds.
   - Precedence: explicit file -> environment variables (`AIOS_DISTRO_*`) -> built-in defaults.
   - Strict size bounds on configuration files (max 64 KiB).
2. **`base_image_config.rs` (`ImageBuildConfig`)**:
   - Manages build directories, output artifact directories, default target IDs, execution timeouts, max image sizes, and compression levels.
   - Precedence: `--config` file -> environment variables (`AIOS_IMAGE_*`) -> embedded defaults.
   - Explicit invariant validation (`CF1..CF6`) for all parameters.
3. **`ledger_config.rs` (`LedgerConfig`)**:
   - Manages lock timeouts and file paths with environment overrides (`AIOS_LEDGER_*`).

---

## 3. Authoritative Sources & Citations
1. **Debian Policy Manual (§3, §7)** & **`apt.conf(5)`**:
   - Defines system configuration hierarchies: `/etc/apt/apt.conf.d/`, state directories (`/var/lib/dpkg/status`), and default release preferences (`APT::Default-Release`).
   - Citation: Debian Policy Manual v4.7.0, *System Configuration and Binary Package Invariants*.
2. **Alpine Linux Package Management (`apk(8)`)**:
   - Defines configuration structure in `/etc/apk/`: repository sources (`/etc/apk/repositories`), world targets (`/etc/apk/world`), and cache directories (`/var/cache/apk`).
   - Citation: Alpine Linux Documentation, *APK Architecture and Configuration Guide*.
3. **XDG Base Directory Specification**:
   - Standardizes user and system data storage: `$XDG_CONFIG_HOME` (`config/` or `~/.config/`) for settings, and `$XDG_DATA_HOME` (`.aios/` or `~/.local/share/`) for persistent state stores.
   - Citation: freedesktop.org XDG Base Directory Specification v0.8.
4. **ADR-0035 (AIOS Audit & Capability Governance)**:
   - Requires deterministic configuration resolution, explicit error envelopes upon invalid settings, and immutable audit row emission whenever security-relevant settings are parsed or updated.

---

## 4. Facts vs. Assumptions

| Item | Status | Details |
|---|---|---|
| In-memory store limits | **Fact** | `PackageStore::load_from_path` currently hardcodes a 10 MiB byte limit and 10,000 package count limit. |
| CLI `--store` flag | **Fact** | `aiosh package` supports `--store <path>`, falling back to in-memory defaults if omitted. |
| MCP tool parameter | **Fact** | All package MCP tools accept optional `store_path` strings bounded to 1,024 characters. |
| Config resolution precedence | **Assumption (to codify)** | File configuration takes precedence over environment variables, which take precedence over built-in defaults: `Config File > Environment Variables > Embedded Defaults`. |
| Default store location | **Assumption (to codify)** | The canonical default store path should be `.aios/packages.json` (consistent with `.aios/image_store.json` and `.aios/handoff_store.json`). |
| Auto-persistence behavior | **Assumption (to codify)** | When configured with `auto_persist = true`, mutations in `apply` automatically save to the configured store path even if `--store` is not explicitly passed. |

---

## 5. Unknowns & Decisions Needed

### Decision 1: Default Store Path and Filename
- **Option A**: `.aios/packages.json` (Consistent with other Phase 1 runtime stores like `.aios/handoff_store.json` and `.aios/image_store.json`).
- **Option B**: `config/packages.json` (More prominent, but blends configuration with dynamic state).
- **Recommendation**: Option A (`.aios/packages.json`).

### Decision 2: Environment Variable Prefix
- **Option A**: `AIOS_PACKAGE_*` (e.g. `AIOS_PACKAGE_STORE_PATH`, `AIOS_PACKAGE_DEFAULT_FORMAT`, `AIOS_PACKAGE_MAX_ENTITIES`).
- **Option B**: `AIOSH_PACKAGE_*`.
- **Recommendation**: Option A (`AIOS_PACKAGE_*`) to maintain parity with `AIOS_IMAGE_*` and `AIOS_DISTRO_*`.

### Decision 3: Configuration Schema Fields
The configuration struct `PackageConfig` should encompass:
1. `store_path: PathBuf` (Default: `.aios/packages.json`).
2. `default_format: PackageFormat` (Default: `PackageFormat::Deb`).
3. `max_store_size_bytes: u64` (Default: `10 * 1024 * 1024` = 10 MiB; valid range: 64 KiB .. 100 MiB).
4. `max_entity_count: usize` (Default: `10,000`; valid range: 10 .. 100,000).
5. `auto_persist: bool` (Default: `false`, explicit persistence preferred).
6. `allowed_repositories: Vec<String>` (Default: `["https://deb.debian.org/debian", "https://dl-cdn.alpinelinux.org/alpine/v3.19/main"]`).

---

## 6. Acceptance Criteria Verification
- [x] Authoritative sources collected and cited.
- [x] Facts separated from assumptions.
- [x] No source code modified during research.
- [x] Explicit decisions listed for specification task (`T-01242`).
