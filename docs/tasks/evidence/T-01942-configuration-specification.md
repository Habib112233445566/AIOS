# Task Evidence: T-01942 - System Update / Configuration: Specification

- **Task**: `T-01942`
- **Sub-Epic**: `Sub-Epic 5: Configuration & Policy`
- **Status**: Completed
- **Date**: 2026-09-20

## 1. Specification Contract: `SystemUpdateConfig`

### 1.1 Data Model
```rust
pub const MAX_UPDATE_CONFIG_FILE_BYTES: u64 = 1_048_576; // 1 MB
pub const DEFAULT_UPDATE_CONFIG_PATH: &str = ".aios/system_update.json";
pub const DEFAULT_UPDATE_STATE_DIR: &str = "/var/lib/aiosh/updates";
pub const DEFAULT_UPDATE_STAGING_DIR: &str = "/var/lib/aiosh/updates/staging";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemUpdateConfig {
    pub state_dir: PathBuf,
    pub staging_dir: PathBuf,
    pub default_channel: UpdateChannel,
    pub check_interval_secs: u64,
    pub allow_auto_apply: bool,
    pub auto_rollback_on_failure: bool,
    pub max_payload_bytes: u64,
    pub min_free_space_bytes: u64,
    pub max_download_rate_bps: Option<u64>,
    pub trusted_keys: Vec<String>,
}
```

### 1.2 Default Values
- `state_dir`: `/var/lib/aiosh/updates`
- `staging_dir`: `/var/lib/aiosh/updates/staging`
- `default_channel`: `UpdateChannel::Stable`
- `check_interval_secs`: `86400` (24 hours)
- `allow_auto_apply`: `false` (manual confirmation required by default)
- `auto_rollback_on_failure`: `true` (automatic recovery enabled)
- `max_payload_bytes`: `MAX_UPDATE_PAYLOAD_SIZE` (10 GB)
- `min_free_space_bytes`: `1_073_741_824` (1 GB minimum safety reserve)
- `max_download_rate_bps`: `None` (unthrottled)
- `trusted_keys`: `vec![]`

### 1.3 Validation Invariants (`validate()`)
1. **`UCONF1` (Path Hygiene)**:
   - `state_dir` and `staging_dir` must be valid UTF-8, non-empty, length $\le 1024$, zero control characters, zero `..` parent directory traversal components.
2. **`UCONF2` (Resource & Interval Bounds)**:
   - `check_interval_secs`: $60 \le t \le 2_592_000$ (1 minute to 30 days).
   - `max_payload_bytes`: $1_048_576 \le \text{bytes} \le 10_737_418_240$ (1 MB to 10 GB).
   - `min_free_space_bytes`: $\le 107_374_182_400$ (100 GB).
3. **`UCONF3` (Key Bounds)**:
   - `trusted_keys`: maximum of 32 keys, each key string $\le 256$ characters and free of control characters.

### 1.4 Environment Ingestion (`from_env()`)
Overrides default fields if defined:
- `AIOSH_UPDATE_STATE_DIR`: sets `state_dir`
- `AIOSH_UPDATE_STAGING_DIR`: sets `staging_dir`
- `AIOSH_UPDATE_CHANNEL`: parses `stable`, `beta`, `nightly` into `default_channel`
- `AIOSH_UPDATE_CHECK_INTERVAL_SECS`: parses `u64` into `check_interval_secs`
- `AIOSH_UPDATE_AUTO_APPLY`: parses boolean (`true`, `1`, `yes`) into `allow_auto_apply`
- `AIOSH_UPDATE_AUTO_ROLLBACK`: parses boolean (`false`, `0`, `no`) into `auto_rollback_on_failure`
- `AIOSH_UPDATE_MAX_PAYLOAD_BYTES`: parses `u64` into `max_payload_bytes`

### 1.5 Persistence (`from_file()`, `save_to_file()`)
- `from_file(path)`:
  - Rejects symlink paths via `symlink_metadata()`.
  - Rejects files $> 1\text{MB}$.
  - Reads bytes, deserializes JSON, and calls `validate()`.
- `save_to_file(path)`:
  - Validates config.
  - Serializes to JSON.
  - Writes to `<path>.tmp.<pid>`, fsyncs, and atomically renames.

### 1.6 Interoperability
- Implements `to_service_config(&self) -> SystemUpdateServiceConfig` for zero-overhead bridge to `SystemUpdateService`.
