# Specification: Secrets Handling Configuration (SPEC-SECRETS-CONFIG)

- **Status**: APPROVED
- **Date**: 2026-10-02
- **Subsystem**: Phase 2 Security Kernel & PEP Fabric / Secrets Handling Configuration
- **Binding**: ADR-0035 §F-2

## 1. Data Structure Specification

```rust
pub struct SecretConfig {
    pub version: String,
    pub store_path: PathBuf,
    pub max_secrets_capacity: usize,
    pub max_payload_bytes: usize,
    pub max_store_file_bytes: u64,
    pub require_expose_flag: bool,
    pub enforce_scope_containment: bool,
    pub audit_all_reads: bool,
    pub audit_all_writes: bool,
}
```

### Numerical Boundaries & Defaults
| Parameter | Default | Minimum | Maximum | Error Code |
|:---|:---|:---|:---|:---|
| `version` | `"1.0.0"` | Non-empty `"1."` | - | `SECCONF_ERR_VALIDATION` |
| `store_path` | `"target/secrets_vault.json"` | No `..` traversal | - | `SECCONF_ERR_VALIDATION` |
| `max_secrets_capacity` | `1024` | `1` | `16384` | `SECCONF_ERR_BOUNDS` |
| `max_payload_bytes` | `65536` (64 KiB) | `1` | `1048576` (1 MiB) | `SECCONF_ERR_BOUNDS` |
| `max_store_file_bytes` | `1048576` (1 MiB) | `4096` (4 KiB) | `16777216` (16 MiB) | `SECCONF_ERR_BOUNDS` |
| `require_expose_flag` | `true` | `false` | `true` | - |
| `enforce_scope_containment` | `true` | `false` | `true` | - |
| `audit_all_reads` | `true` | `false` | `true` | - |
| `audit_all_writes` | `true` | `false` | `true` | - |

## 2. Environment Variable Overrides
- `AIOS_SECRETS_CONFIG`: File path to custom `secrets_config.json` (max 1024 bytes, traversal prohibited).
- `AIOS_SECRETS_STORE`: Target path for secrets persistence store (max 1024 bytes, traversal prohibited).
- `AIOS_SECRETS_MAX_CAPACITY`: Numerical override for `max_secrets_capacity` (`1..=16384`).
- `AIOS_SECRETS_MAX_PAYLOAD`: Numerical override for `max_payload_bytes` (`1..=1048576`).
- `AIOS_SECRETS_MAX_STORE_FILE`: Numerical override for `max_store_file_bytes` (`4096..=16777216`).
- `AIOS_SECRETS_REQUIRE_EXPOSE`: Boolean override for `require_expose_flag` (`"true"` / `"false"`).
- `AIOS_SECRETS_AUDIT_ALL`: Boolean override for `audit_all_reads` and `audit_all_writes`.

## 3. Interfaces
### 3.1 CLI Interface
```bash
aiosh secret config [show|check] [--config <path>] [--json]
```
- `show`: Displays active resolved configuration parameters in human-readable table or structured JSON.
- `check`: Validates configuration against all bounds and constraints.

### 3.2 MCP Tool Interface
```json
{
  "name": "aios.secret.config",
  "description": "Inspect or validate Secrets Handling configuration parameters and bounds",
  "inputSchema": {
    "type": "object",
    "properties": {
      "action": {"type": "string", "enum": ["show", "check"]},
      "config_path": {"type": "string"}
    },
    "additionalProperties": false
  }
}
```

## 4. Invariants
- **SECCONF1**: File size of config file must not exceed 64 KiB (`65,536` bytes).
- **SECCONF2**: Store path must not contain `..` directory traversal tokens, control characters, or exceed 1024 bytes.
- **SECCONF3**: Out-of-bounds parameters reject loading fail-closed with explicit error codes (`SECCONF_ERR_BOUNDS`).
- **SECCONF4**: Configuration saving uses atomic file rename with temporary file cleanup.
- **SECCONF5**: Symlinks are rejected fail-closed via `symlink_metadata` inspection.
- **SECCONF6**: Secure-by-default posture requires expose flags and comprehensive audit logging.

## 5. Test Suite Catalog
- Unit Tests: `code/aiosh-rust/aiosh-core/tests/test_secret_config.rs` (5 tests).
- Integration Tests: `code/aiosh-rust/aiosh-core/tests/test_secret_config_integration.rs` (2 tests).
- CLI Surface Tests: `code/aiosh-rust/aiosh-cli` (`test_secret_cli_config`).
- MCP Tool Tests: `code/aiosh-rust/aiosh-mcp` (`test_mcp_secret_tools_execution`).

