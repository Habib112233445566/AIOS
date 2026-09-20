# Task Evidence: T-01948 - System Update / Configuration: Hardening

- **Task**: `T-01948`
- **Sub-Epic**: `Sub-Epic 5: Configuration & Policy`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Hardening Work
Implemented defensive hardening across the System Update Configuration subsystem in `code/aiosh-rust/aiosh-core/src/system_update_config.rs`:

1. **Environment Ingestion Pre-Filter**:
   - `from_env()`: Validates environment variable overrides (`AIOSH_UPDATE_STATE_DIR`, `AIOSH_UPDATE_STAGING_DIR`) before assigning to `PathBuf`. Discards inputs that exceed 1024 characters, contain control characters, or contain `..` parent traversal components.
   - Bounded numerical parsing: `AIOSH_UPDATE_CHECK_INTERVAL_SECS` is constrained to $60 \le t \le 2_592_000$, and `AIOSH_UPDATE_MAX_PAYLOAD_BYTES` is constrained to $1\text{MB} \le \text{bytes} \le 10\text{GB}$.

2. **Resource & Bandwidth Bounds**:
   - `min_free_space_bytes`: Enforces that free space reservation cannot be set below 1MB (`1_048_576` bytes) or above 100GB, preventing disk starvation and unreasonable configuration.
   - `max_download_rate_bps`: Enforces bounds between 1 bps and 10 Gbps when configured.

3. **Atomic File Persistence & Temporary File Cleanup**:
   - `save_to_file()`: Unlinks any stale `<path>.tmp.<pid>` before opening write handles.
   - If rename or serialization fails, the temporary file is unlinked immediately to prevent residue buildup.

4. **Symlink and File Bounds**:
   - `from_file()` rejects any symlink configuration file via `symlink_metadata()` and caps file size to 1MB.

## Verification
- Unit test suite: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_config`.
- Integration smoke suite: `python code/aiosh-mcp/tests/test_system_update_config_smoke.py`.
