# T-00728 — Secrets & Access Hygiene / core service: Hardening

## 1. Hardening Scope
This task verifies defensive hardening mechanisms across the secrets scanning service in `code/aiosh-rust/aiosh-core/src/secrets_service.rs`.

## 2. Hardening Measures
- **Non-Existent Directory Protection**: `scan_workspace_for_secrets` returns an explicit `Err(String)` error envelope when passed non-existent paths (`test_scan_workspace_nonexistent`), preventing panic conditions.
- **Unreadable File Resilience**: Unreadable files or I/O errors are skipped gracefully, allowing full workspace scanning to continue without partial aborts.
- **Binary Content Elimination**: Null-byte detection skips non-text files, preventing scanning overhead on binary blobs.
- **Line Length and File Size Caps**: Line scanning is clamped to 4096 bytes and file sizes to 16 MiB.

## 3. Test Verification Output
```text
running 7 tests
test secrets_service::tests::test_scan_file_aws_key_and_ghp ... ok
test secrets_service::tests::test_scan_binary_file_skipped ... ok
test secrets_service::tests::test_scan_file_clean ... ok
test secrets_service::tests::test_scan_file_password_in_config ... ok
test secrets_service::tests::test_scan_file_private_key ... ok
test secrets_service::tests::test_scan_workspace_nonexistent ... ok
test secrets_service::tests::test_scan_workspace ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 181 filtered out; finished in 0.20s
```
