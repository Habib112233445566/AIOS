# T-00957 — Agent Handoff Protocol / Configuration: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-1: Oversized Config File Resource Exhaustion (Zip/JSON Bomb)
- **Threat**: Attacker specifies a multi-gigabyte file as config file to exhaust process memory.
- **Mitigation**: `from_file` strictly enforces `MAX_CONFIG_FILE_BYTES` (64 KiB) limit before reading file content into memory.

### AS-2: Negative / Zero TTL Handoff Expiration Bypass
- **Threat**: Setting `default_ttl_seconds: 0` causing immediate purge or integer underflow.
- **Mitigation**: `validate()` explicitly requires `default_ttl_seconds >= 1`.

### AS-3: Store Allocation Exhaustion
- **Threat**: Attacker sets `max_store_bytes` to unbounded values (e.g., terabytes) or tiny values (causing denial of service).
- **Mitigation**: Bounds enforced strictly between `MIN_STORE_BYTES` (16 KiB) and `MAX_STORE_BYTES` (64 MiB).
