# Task Evidence: T-02644 — Secrets Handling / configuration: Implementation

## 1. Task Metadata
- **Task ID**: `T-02644`
- **Sub-Epic**: Phase 2 — Security Kernel & PEP Fabric / Secrets Handling / configuration
- **Type**: Implementation
- **Status**: Completed
- **Date**: 2026-10-03

## 2. Work Delivered
1. **Core Service Configuration Integration (`SecretService`)**:
   - Integrated `SecretConfig` into `aiosh-core/src/secret_service.rs`.
   - Added `SecretService::new_with_config(config: SecretConfig)`.
   - Exposed `service.config()` and `service.config_mut()`.
   - Enforced maximum secret payload size bound: `entry.value.as_bytes().len() <= config.max_payload_bytes`.
   - Enforced maximum vault entries capacity: `self.entries.len() < config.max_secrets_capacity`.
   - Added `SecretService::load_from_path_with_config(path, config)` with file size limit validation based on `config.max_store_file_bytes`.

2. **CLI Surface (`aiosh secret config`)**:
   - Implemented `aiosh secret config <show|check> [--config <PATH>] [--json]` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Added sanitization and traversal prevention checks on `--config` parameter.
   - Added `test_secret_cli_config` covering show, check, json output, invalid actions, and path traversal rejection.

3. **MCP Surface (`aios.secret.config`)**:
   - Registered tool `aios.secret.config` in tool definitions list in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Supported actions: `show` and `check` with optional `config_path`.
   - Added path traversal prevention fail-closed checks.
   - Updated `test_mcp_secret_tools_execution` validating advertisement, execution, and traversal rejection.

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml --workspace` -> Passed (0 errors, 0 warnings).
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli secret_cli_tests` -> 4 tests passed.
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp test_mcp_secret_tools_execution` -> 1 test passed.
