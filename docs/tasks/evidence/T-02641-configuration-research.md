# Task Evidence: T-02641 — Secrets Handling Configuration Research

## 1. Objective
Establish facts, constraints, and prior art for the Configuration subsystem of Secrets Handling in `code/aiosh-rust/aiosh-core`.

## 2. Findings: Facts vs. Assumptions

### Authoritative Facts
1. **Existing Subsystem Configuration Patterns**:
   - `privilege_config.rs`, `sandbox_config.rs`, and `system_update_config.rs` follow a consistent canonical architecture:
     - Strongly typed Rust struct deriving `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, `Eq`.
     - File-backed configuration with a 64 KiB maximum file size limit.
     - Environment variable overrides for production/containerized environments.
     - Path traversal prevention (`..` rejection).
     - Structural boundary validation enforcing numerical floors and ceilings.
2. **Current Hardcoded Secrets Parameters**:
   - Currently, `SecretService` defaults to 1,024 secret capacity, 64 KiB payload limit, and 1 MiB store file size limit.
   - Centralizing these limits in `SecretConfig` allows operators and CI to customize vault bounds cleanly without recompiling.
3. **CLI & MCP Parity**:
   - Every subsystem in Phase 2 exposes its configuration through:
     - CLI: `aiosh secret config [show|check]`
     - MCP: `aios.secret.config`

### Assumptions & Decisions
- **Config File Location**: Default is `.aios/secrets_config.json`, overridden by `AIOS_SECRETS_CONFIG` environment variable.
- **Store Path**: Default is `target/secrets_vault.json`, overridden by `AIOS_SECRETS_STORE`.
- **Validation Rules**:
  - `max_secrets_capacity`: 1..16,384 (default 1,024).
  - `max_payload_bytes`: 1..1,048,576 (default 65,536).
  - `max_store_file_bytes`: 4,096..16,777,216 (default 1,048,576).
  - `require_expose_flag`: boolean (default true).
  - `store_path`: must not contain directory traversal `..`.

## 3. Decisions for Specification (T-02642)
- Formulate `SPEC-SECRETS-CONFIG.md` detailing struct fields, JSON serialization format, validation error codes (`SECCONF_ERR_*`), environment variable bindings, and CLI/MCP interfaces.
