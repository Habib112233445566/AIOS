# T-00724 — Secrets & Access Hygiene / core service: Implementation

## 1. Implementation Deliverables
- Implemented `scan_file_for_secrets` in `code/aiosh-rust/aiosh-core/src/secrets_service.rs` supporting:
  - `SEC-001`: Private Key detection (`-----BEGIN ... PRIVATE KEY-----`).
  - `SEC-002`: AWS Access Key ID detection (`AKIA...`).
  - `SEC-003`: GitHub Personal Access Token detection (`ghp_...`).
  - `SEC-004`: Generic API / Bearer Token assignment detection.
  - `SEC-005`: Hardcoded password / key assignment detection.
- Implemented `scan_workspace_for_secrets` with recursive directory traversal and `.git`, `target`, `node_modules`, `.venv`, `dist` skipping.
- Added comprehensive unit tests in `secrets_service::tests`.

## 2. Test Verification Output
```text
running 4 tests
test secrets_service::tests::test_scan_file_aws_key_and_ghp ... ok
test secrets_service::tests::test_scan_file_private_key ... ok
test secrets_service::tests::test_scan_workspace ... ok
test secrets_service::tests::test_scan_file_clean ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 181 filtered out; finished in 0.05s
```
