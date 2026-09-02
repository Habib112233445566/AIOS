# T-00257 — Configuration: Security Review

## Abuse Scenarios
1. **Config file path injection**: `$AIOSH_RELEASE_CONFIG` could point to `/etc/shadow`. Mitigated: `load_config` only reads JSON, never executes. Reading a non-JSON file raises `ValueError`.
2. **Config value overflow**: Setting `max_file_size_bytes` to extreme values. Mitigated: Clamped to [1MB, 10GB] range.
3. **Config file write**: Config is read-only. No function writes to the config file. No audit row needed since loading config is not a state-changing action.
4. **TOCTOU on config reload**: Each call to `generate_release`/`create_backup` re-reads config. An attacker who modifies config mid-operation could change limits. Mitigated: acceptable risk — config changes require filesystem access which implies already-compromised operator level.

No policy bypass found.
