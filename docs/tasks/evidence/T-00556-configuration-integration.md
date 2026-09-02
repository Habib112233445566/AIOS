# T-00556 — Evidence & Audit Trail / configuration: Integration

## 1. Integration Scope
This task integrates `EvidenceConfig` into `aiosh-core::evidence_service`, wiring configuration loading (`compute_file_sha256_with_config`, `EvidenceConfig::from_env`) and validating that environment variable overrides and configuration files dynamically govern file read limits and directories across the service layer.

## 2. Integration Points
- `compute_file_sha256_with_config(path, config)`: Enforces `config.max_file_bytes` dynamically.
- `compute_file_sha256(path)`: Loads configuration via `EvidenceConfig::from_env()`.
- Default repo configuration `config/evidence.config.json` is verified during test execution.

## 3. Verification
- `cargo test -p aiosh-core evidence_service::tests` -> 6/6 tests pass.
- `cargo test -p aiosh-core evidence_config::tests` -> 8/8 tests pass.
