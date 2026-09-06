# T-01349: Init & Service Supervision - Configuration: Documentation

## Metadata
- **Task ID:** `T-01349`
- **Subsystem:** `code/aiosh-rust`
- **Component:** Init & Service Supervision Configuration Documentation
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Documentation Overview

Documented the Init & Service Supervision Configuration Subsystem (`aiosh-core::service_config`) in `docs/README.md`.

### Delivered Capabilities:
- **`ServiceConfig` Data Structure**: `store_path`, `default_timeout_start_secs`, `default_timeout_stop_secs`, `max_store_size_bytes`, `max_entity_count`, `auto_persist`, `restart_backoff_secs`, `max_restart_burst`.
- **Invariants `SC1..SC7`**: Path validity, execution timeout limits $[1..3600]$s, store size bounds $[64\text{ KiB}..100\text{ MiB}]$, entity bounds $[10..100,000]$, restart throttling bounds $[1..300]$s / $[1..50]$ burst, and config file cap (64 KiB).
- **Precedence Hierarchy (`SC6`)**: File (`--config`) > Environment (`AIOS_SERVICE_*`) > Embedded safe defaults.
- **Operator Command**: `aiosh service config [--config <path>] [--json]`.
- **Autonomous Agent MCP Tool**: `aios.service.config`.

---

## 2. Copy-Pasteable Usage Examples

### Operator CLI Example:
```bash
# View active service configuration in human-readable table
aiosh service config

# View active service configuration in JSON format
aiosh service config --json

# Load configuration from an explicit custom path
aiosh service config --config ./custom_service_config.json --json
```

### Autonomous Agent MCP JSON-RPC Example:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.service.config",
    "arguments": {}
  }
}
```

---

## 3. Constraints & Known Limitations (Honest)
1. **Configuration File Size Ceiling**: Configuration files on disk must not exceed 64 KiB (65,536 bytes). Oversized files are rejected immediately with `SC7 violation` to prevent resource exhaustion.
2. **Timeout Boundaries**: Both startup and shutdown timeouts must be within $[1 \dots 3,600]$ seconds. Zero-second and indefinite timeouts are disallowed to prevent supervision hangs.
3. **Restart Throttling**: Restart delays must be within $[1 \dots 300]$ seconds with a burst cap between 1 and 50 attempts to protect system stability against fork loops.
4. **Store Sizing**: Maximum store file size is bounded to $[64\text{ KiB} \dots 100\text{ MiB}]$ with $[10 \dots 100,000]$ service entities.

---

## 4. Linked Evidence Chain
- Research: `docs/tasks/evidence/T-01341-configuration-research.md`
- Specification: `docs/tasks/evidence/T-01342-configuration-specification.md`
- Scaffold: `docs/tasks/evidence/T-01343-configuration-scaffold.md`
- Implementation: `docs/tasks/evidence/T-01344-configuration-implementation.md`
- Unit Tests: `docs/tasks/evidence/T-01345-configuration-unit-test.md`
- Integration: `docs/tasks/evidence/T-01346-configuration-integration.md`
- Security Review: `docs/tasks/evidence/T-01347-configuration-security-review.md`
- Hardening: `docs/tasks/evidence/T-01348-configuration-hardening.md`
