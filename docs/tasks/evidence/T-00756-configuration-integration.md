# T-00756 — Secrets & Access Hygiene / configuration: Integration

## 1. Integration Deliverables
- Integrated `SecretsConfig` into `aiosh-cli::cmd_secrets`:
  - `--config <path>` flag support to load custom scanning rules and ignored directories.
  - Automatic fallback to environment variable `AIOS_SECRETS_CONFIG` and `docs/secrets_config.json`.
- Integrated `scan_workspace_with_config` into `aiosh-core::secrets_service`.
- Verified passing CLI integration tests in `aiosh-cli::task_cli_tests::test_cmd_secrets_scan_and_check`.
