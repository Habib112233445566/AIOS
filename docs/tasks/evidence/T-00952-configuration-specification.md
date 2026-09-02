# T-00952 — Agent Handoff Protocol / Configuration: Specification

## 1. Schema & Defaults

```json
{
  "max_store_bytes": 1048576,
  "default_priority": "Normal",
  "default_ttl_seconds": 86400,
  "allow_auto_accept": false,
  "store_path": null
}
```

## 2. Invariants & Validation Rules
- `max_store_bytes`: Must be between 16,384 bytes (16 KiB) and 67,108,864 bytes (64 MiB).
- `default_ttl_seconds`: Must be greater than 0 (minimum 1 second).
- File read bound: Config files larger than 65,536 bytes (64 KiB) are rejected.
- Loading order: Path explicitly passed $\to$ `AIOSH_HANDOFF_CONFIG` env var $\to$ `docs/handoff_config.json` $\to$ default struct in memory.
