# T-00252 — Release Packaging & Backup / configuration: Specification

## Config Schema (`config/release.json`)
```json
{
  "max_file_size_bytes": 2147483648,
  "default_components": ["core"],
  "output_dir": "output/release",
  "backup_defaults": {
    "include_audit": true,
    "include_memory": false
  }
}
```

## Loading Contract
- **Python**: `release_config.load_config(path=None) -> dict` — reads `$AIOSH_RELEASE_CONFIG` or falls back to `config/release.json`. Returns validated dict.
- **Rust**: `release_config::load_config(path: Option<&str>) -> ReleaseConfig` — mirrors Python.
- **Failure**: If file missing, returns hardcoded defaults silently (no crash). If file malformed, returns error.

## Persistence Effects
- Config is read-only. No audit row emitted on config load (it's not a state-changing action).

## Error Cases
- Missing file → use defaults.
- Malformed JSON → return `Err` / raise `ValueError`.
- `max_file_size_bytes` out of range (< 1MB or > 10GB) → clamp to bounds.
