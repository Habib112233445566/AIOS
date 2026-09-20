# Task Evidence: T-01941 - System Update / Configuration: Research

- **Task**: `T-01941`
- **Sub-Epic**: `Sub-Epic 5: Configuration & Policy`
- **Status**: Completed
- **Date**: 2026-09-20

## 1. Executive Summary & Context
Researched the configuration and policy subsystem requirements for the System Update Mechanism (`aiosh-core::system_update_config`). The objective is to design a robust, bounded configuration structure (`SystemUpdateConfig`) that integrates with `SystemUpdateService`, follows existing config patterns in `aiosh-core` (`network_config.rs`, `release_config.rs`), and provides environment variable ingestion and atomic file persistence.

---

## 2. Facts vs. Assumptions

### Authoritative Facts:
1. **Existing Service Configuration**: `aiosh-core::system_update_service::SystemUpdateServiceConfig` currently only specifies `state_dir`, `staging_dir`, `max_payload_bytes`, and `auto_rollback_on_failure`.
2. **Prior Art in `aiosh-core`**:
   - `network_config.rs`, `hardware_config.rs`, and `release_config.rs` implement:
     - `default()` for safe, secure-by-default initial values.
     - `validate()` enforcing invariants `*CONF1..*CONF6` (path hygiene, bounds, value ranges).
     - `from_env()` supporting targeted environment variable overrides (`AIOSH_*`).
     - `from_file()` enforcing a 1MB file size cap and rejecting symlinks.
     - `save_to_file()` using atomic file persistence with `.tmp` and atomic rename.
3. **Upstream Standards**:
   - `systemd-sysupdate` and `RAUC` define update channels, check intervals, storage safety thresholds, and reboot policies.
   - Dual-bank A/B systems mandate safe fallback defaults where an unconfigured system defaults to non-destructive manual apply with automatic rollback.

### Assumptions:
1. Canonical configuration file path is `/etc/aiosh/system_update.json` (system) or `.aios/system_update.json` (local).
2. Environment variable prefix is `AIOSH_UPDATE_*` (`AIOSH_UPDATE_CONFIG_PATH`, `AIOSH_UPDATE_STATE_DIR`, `AIOSH_UPDATE_STAGING_DIR`, `AIOSH_UPDATE_CHANNEL`, `AIOSH_UPDATE_AUTO_APPLY`, `AIOSH_UPDATE_AUTO_ROLLBACK`, `AIOSH_UPDATE_CHECK_INTERVAL_SECS`).
3. Serialization format is JSON, bounded to 1MB (`MAX_CONFIG_FILE_BYTES = 1_048_576`).

---

## 3. Configuration Invariants (UCONF1..UCONF6)
- **`UCONF1` (Path Hygiene)**: All path fields (`state_dir`, `staging_dir`, `log_dir`) must be non-empty UTF-8, $\le 1024$ chars, free of control characters, and free of `..` parent directory traversal components.
- **`UCONF2` (Resource & Size Bounds)**: `max_payload_bytes` capped at 10GB (`MAX_UPDATE_PAYLOAD_SIZE`), `min_free_space_bytes` bounded, and configuration file size capped at 1MB.
- **`UCONF3` (Channel & Target Validity)**: `default_channel` must be a recognized variant (`stable`, `beta`, `nightly`).
- **`UCONF4` (Environment Ingestion)**: Environment variables override file and default values deterministically with input validation.
- **`UCONF5` (Atomic & Bounded Persistence)**: Saving configuration writes to `.tmp` file, fsyncs, and atomically renames.
- **`UCONF6` (Fail-Safe Default Fallback)**: If configuration file is missing or corrupted, the service falls back to default configuration without panic.

---

## 4. Decisions Needed Before Specification
1. **Auto-Apply Default**: Explicitly set to `false` to avoid unintentional production restarts.
2. **Auto-Rollback Default**: Explicitly set to `true` to ensure system recoverability.
3. **Integration with `SystemUpdateService`**: `SystemUpdateServiceConfig` can be constructed from or backed by `SystemUpdateConfig`.
