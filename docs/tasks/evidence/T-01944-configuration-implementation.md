# Task Evidence: T-01944 - System Update / Configuration: Implementation

- **Task**: `T-01944`
- **Sub-Epic**: `Sub-Epic 5: Configuration & Policy`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Implementation Work
Implemented the complete, production-ready System Update Configuration subsystem in `code/aiosh-rust/aiosh-core/src/system_update_config.rs`:

1. **Data Model**:
   - `SystemUpdateConfig`: captures `state_dir`, `staging_dir`, `default_channel`, `check_interval_secs`, `allow_auto_apply`, `auto_rollback_on_failure`, `max_payload_bytes`, `min_free_space_bytes`, `max_download_rate_bps`, and `trusted_keys`.
   - `Default`: defaults to `/var/lib/aiosh/updates`, `/var/lib/aiosh/updates/staging`, `UpdateChannel::Stable`, 24h interval, manual apply (`false`), auto rollback (`true`), 10GB max payload, 1GB minimum reserve, unthrottled, empty trusted keys.

2. **Validation (`validate`)**:
   - `UCONF1`: Path hygiene for `state_dir` and `staging_dir` (length $\le 1024$, non-empty UTF-8, zero control characters, zero `..` traversal).
   - `UCONF2`: Bounds validation:
     - `check_interval_secs`: between 60 and 2,592,000 seconds.
     - `max_payload_bytes`: between 1MB and 10GB.
     - `min_free_space_bytes`: capped at 100GB.
   - `UCONF3`: Trusted keys: maximum 32 keys, each non-empty, $\le 256$ chars, no control chars.

3. **Environment Ingestion (`from_env`)**:
   - `UCONF4`: Ingests `AIOSH_UPDATE_STATE_DIR`, `AIOSH_UPDATE_STAGING_DIR`, `AIOSH_UPDATE_CHANNEL`, `AIOSH_UPDATE_CHECK_INTERVAL_SECS`, `AIOSH_UPDATE_AUTO_APPLY`, `AIOSH_UPDATE_AUTO_ROLLBACK`, `AIOSH_UPDATE_MAX_PAYLOAD_BYTES`.

4. **Persistence (`from_file`, `save_to_file`)**:
   - `UCONF5`: Atomic persistence via `<path>.tmp.<pid>` and rename.
   - `UCONF6`: Symlink rejection via `symlink_metadata()` and 1MB size limit check (`MAX_UPDATE_CONFIG_FILE_BYTES = 1_048_576`).

5. **Interoperability**:
   - `to_service_config(&self)`: converts into `SystemUpdateServiceConfig` for `SystemUpdateService`.

## Verification
- Code successfully compiled and verified in `code/aiosh-rust/aiosh-core`.
