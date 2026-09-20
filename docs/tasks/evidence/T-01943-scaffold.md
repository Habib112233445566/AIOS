# Task Evidence: T-01943 - System Update / Configuration: Scaffold

- **Task**: `T-01943`
- **Sub-Epic**: `Sub-Epic 5: Configuration & Policy`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Scaffold Work
Created the module skeleton and interface definitions for System Update Configuration in `code/aiosh-rust/aiosh-core/src/system_update_config.rs`:
- Defined `SystemUpdateConfig` struct with typed fields:
  - `state_dir: PathBuf`
  - `staging_dir: PathBuf`
  - `default_channel: UpdateChannel`
  - `check_interval_secs: u64`
  - `allow_auto_apply: bool`
  - `auto_rollback_on_failure: bool`
  - `max_payload_bytes: u64`
  - `min_free_space_bytes: u64`
  - `max_download_rate_bps: Option<u64>`
  - `trusted_keys: Vec<String>`
- Defined constants:
  - `MAX_UPDATE_CONFIG_FILE_BYTES: u64 = 1_048_576` (1 MB)
  - `DEFAULT_UPDATE_CONFIG_PATH = ".aios/system_update.json"`
  - `DEFAULT_UPDATE_STATE_DIR = "/var/lib/aiosh/updates"`
  - `DEFAULT_UPDATE_STAGING_DIR = "/var/lib/aiosh/updates/staging"`
  - `UCONF_VALIDATION_ERROR = "UCONF_VALIDATION_ERROR"`
- Implemented trait and method signatures:
  - `Default for SystemUpdateConfig`
  - `validate(&self) -> Result<(), String>`
  - `from_env() -> Self`
  - `from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>`
  - `save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String>`
  - `to_service_config(&self) -> SystemUpdateServiceConfig`
- Wired `pub mod system_update_config;` and exports into `code/aiosh-rust/aiosh-core/src/lib.rs`.

## Verification
- Checked that `code/aiosh-rust/aiosh-core` compiles cleanly with zero errors.
