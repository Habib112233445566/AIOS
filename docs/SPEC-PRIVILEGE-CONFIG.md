# Specification: Privilege Escalation Prevention Configuration

## 1. Overview
The Privilege Escalation Prevention Configuration Subsystem governs the parameters, resource caps, persistence paths, and security policies of the privilege execution environment in `aiosh-core`.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `PRIVESC_CFG1` | **Bounded Memory & Sizing** | Configuration files must not exceed 64 KiB; active context ceiling is bounded to $[1, 16384]$. |
| `PRIVESC_CFG2` | **Path Traversal Shielding** | Configured `store_path` must not contain directory traversal sequences (`..`). |
| `PRIVESC_CFG3` | **Secure-by-Default** | Default settings enforce transition auditing, grant verification, and baseline User privilege tier. |
| `PRIVESC_CFG4` | **Deterministic Validation** | Schema violations, out-of-bound numerical fields, or invalid versions fail with explicit error codes (`PRIVESCCONF_ERR_*`). |
| `PRIVESC_CFG5` | **Environment Determinism** | Supported `AIOS_PRIVILEGE_*` environment variables override file defaults with boundary clamping. |

---

## 3. Schema & Field Definitions

```json
{
  "version": "1.0.0",
  "store_path": "target/privilege_state.json",
  "default_tier": "user",
  "max_active_contexts": 1024,
  "max_grant_duration_seconds": 3600,
  "max_capabilities_per_context": 32,
  "allow_guest_contexts": true,
  "enforce_grant_signatures": true,
  "audit_all_transitions": true
}
```

### Numerical Boundaries

| Parameter | Type | Minimum | Default | Maximum |
|---|---|---|---|---|
| `max_active_contexts` | usize | 1 | 1024 | 16384 |
| `max_grant_duration_seconds` | u64 | 1 | 3600 | 86400 |
| `max_capabilities_per_context` | usize | 1 | 32 | 64 |
| Config file byte size | u64 | 1 | - | 65536 (64 KiB) |

---

## 4. Environment Variable Overrides

| Environment Variable | Target Parameter | Accepted Values |
|---|---|---|
| `AIOS_PRIVILEGE_CONFIG_PATH` | Path to load config file | Valid file path without `..` |
| `AIOS_PRIVILEGE_STORE` / `AIOS_PRIVILEGE_STORE_PATH` | `store_path` | Valid file path without `..` |
| `AIOS_PRIVILEGE_DEFAULT_TIER` | `default_tier` | `guest`, `user`, `operator`, `admin` (SystemKernel forbidden) |
| `AIOS_PRIVILEGE_MAX_CONTEXTS` | `max_active_contexts` | Integer $1..16384$ |
| `AIOS_PRIVILEGE_MAX_GRANT_DURATION` | `max_grant_duration_seconds` | Integer $1..86400$ |
| `AIOS_PRIVILEGE_AUDIT_ALL` | `audit_all_transitions` | `"1"`, `"true"`, `"0"`, `"false"` |

---

## 5. Operations & Interfaces

### CLI Command
- **Syntax**: `aiosh privilege config [--config <PATH>] [--json]`
- **Description**: Displays active privilege configuration parameters, limits, and defaults.

### MCP Tool
- **Tool Name**: `aios.privilege.config`
- **Input Schema**: `{"type": "object", "properties": {"config_path": {"type": "string"}}, "additionalProperties": false}`
- **Description**: Inspects active Privilege Escalation Prevention configuration over JSON-RPC 2.0.

---

## 6. Automated Test Suites
- In-crate Unit Tests: `code/aiosh-rust/aiosh-core/src/privilege_config.rs` (`tests`)
- Integration Tests: `code/aiosh-rust/aiosh-core/tests/test_privilege_config.rs`
- MCP Config Integration: `code/aiosh-mcp/tests/test_privilege_mcp.py` (`test_mcp_privilege_config`)

