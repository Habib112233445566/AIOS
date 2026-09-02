# T-00764 — Secrets & Access Hygiene / automated tests: Implementation

## 1. Test Suite Implementation Deliverables
- Built test harness in `tools/test_secrets_suites.py` asserting criteria K1..K7:
  - `test_k1_data_model_integrity`: Verifies `SecretSeverity`, `SecretFinding`, and `redact_secret_value`.
  - `test_k2_private_key_scanner`: Verifies `SEC-001` RSA/EC private key detection and binary skipping.
  - `test_k3_api_token_scanner`: Verifies `SEC-002` AWS access keys and `SEC-003` GitHub tokens.
  - `test_k4_config_password_scanner`: Verifies `SEC-005` password assignments and `SEC-004` generic tokens.
  - `test_k5_cli_surface`: Verifies `aiosh secrets scan` and `aiosh secrets check` commands and flags.
  - `test_k6_mcp_surface`: Verifies `aios.secrets.scan` and `aios.secrets.check` JSON-RPC tools.
  - `test_k7_config_suite`: Verifies `SecretsConfig` validation, bounds checking, and roundtrips.
