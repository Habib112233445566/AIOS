# T-00725 — Secrets & Access Hygiene / core service: Unit Test

## 1. Test Scope
This task tests unit behavior for the secret scanning service in `code/aiosh-rust/aiosh-core/src/secrets_service.rs` covering:
- Clean file scanning without false positives (`test_scan_file_clean`).
- Binary file detection and skipping (`test_scan_binary_file_skipped`).
- Private key block detection (`test_scan_file_private_key` / `SEC-001`).
- AWS access key and GitHub PAT detection (`test_scan_file_aws_key_and_ghp` / `SEC-002`, `SEC-003`).
- Hardcoded password in config detection (`test_scan_file_password_in_config` / `SEC-005`).
- Recursive workspace scanning and finding aggregation (`test_scan_workspace` / `SEC-004`).

## 2. Test Verification Output
```text
running 6 tests
test secrets_service::tests::test_scan_file_aws_key_and_ghp ... ok
test secrets_service::tests::test_scan_file_clean ... ok
test secrets_service::tests::test_scan_binary_file_skipped ... ok
test secrets_service::tests::test_scan_file_password_in_config ... ok
test secrets_service::tests::test_scan_file_private_key ... ok
test secrets_service::tests::test_scan_workspace ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 181 filtered out; finished in 0.05s
```
