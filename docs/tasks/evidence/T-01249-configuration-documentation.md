# T-01249: Package Management - Configuration: Documentation

## Metadata
- **Task ID:** `T-01249`
- **Subsystem:** `code/aiosh-rust`
- **Component:** Package Management Configuration Documentation
- **Status:** Complete

## 1. Documentation Overview
Documented the Package Management Configuration Subsystem (`aiosh-core::package_config`) in `docs/README.md` (§8.12).

### Delivered Capabilities:
- **`PackageConfig` Data Structure**: `store_path`, `default_format`, `max_store_size_bytes`, `max_entity_count`, `auto_persist`, `allowed_repositories`.
- **Invariants `PC1..PC6`**: Sizing bounds, entity limits, control-character rejection, and repository TLS security.
- **Precedence Hierarchy**: File (`--config`) > Environment (`AIOS_PACKAGE_*`) > Embedded defaults.
- **Operator Command**: `aiosh package config [--config <path>] [--json]`.
- **Agent MCP Tool**: `aios.package.config`.

## 2. Copy-Pasteable Usage Examples

### Operator CLI Example:
```bash
# View active configuration in human-readable format
aiosh package config

# View active configuration in JSON format
aiosh package config --json

# Load configuration from a custom path
aiosh package config --config ./custom_pkg_config.json --json
```

### Autonomous Agent MCP JSON-RPC Example:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.package.config",
    "arguments": {}
  }
}
```

## 3. Constraints & Known Limitations (Honest)
1. **Plaintext Transport Prohibited**: Insecure HTTP endpoints (`http://`) are strictly rejected by invariant `PC4`. Repositories must use HTTPS (`https://`) or local filesystem mirrors (`file://`).
2. **Configuration File Ceiling**: Files exceeding 64 KiB (65,536 bytes) are rejected with error `PC6 violation` to prevent resource exhaustion attacks.
3. **Store Entity Ceiling**: Package stores are bounded to a maximum of 100,000 entities and 100 MiB on disk.

## 4. Linked Evidence Chain
- Research: `docs/tasks/evidence/T-01241-configuration-research.md`
- Specification: `docs/tasks/evidence/T-01242-configuration-specification.md`
- Scaffold: `docs/tasks/evidence/T-01243-configuration-scaffold.md`
- Implementation: `docs/tasks/evidence/T-01244-configuration-implementation.md`
- Unit Tests: `docs/tasks/evidence/T-01245-configuration-unit-test.md`
- Integration: `docs/tasks/evidence/T-01246-configuration-integration.md`
- Security Review: `docs/tasks/evidence/T-01247-configuration-security-review.md`
- Hardening: `docs/tasks/evidence/T-01248-configuration-hardening.md`
