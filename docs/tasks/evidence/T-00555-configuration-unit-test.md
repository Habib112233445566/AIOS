# T-00555 — Evidence & Audit Trail / configuration: Unit Test

## 1. Unit Test Scope
This task tests the `EvidenceConfig` configuration data model, JSON parsing, validation constraints, file loading limits, environment overrides, and live repository config file parsing in `code/aiosh-rust/aiosh-core/src/evidence_config.rs`.

## 2. Test Cases & Coverage
1. `test_evidence_config_default_is_valid`:
   - Checks default field values (`evidence_dir: "docs/tasks/evidence"`, `max_file_bytes: 16 MiB`, `enforce_checksum: true`).
2. `test_evidence_config_roundtrip_happy`:
   - Validates JSON serialization and deserialization roundtrip.
3. `test_evidence_config_validation_failures`:
   - Asserts errors on empty directory, absolute path (`/etc/passwd`), path traversal (`..`), 0 byte limit, >64 MiB limit, empty extensions, and missing dot in extension.
4. `test_evidence_config_from_path_and_missing`:
   - Checks successful loading from file and error on missing path.
5. `test_evidence_config_from_env_fallback`:
   - Checks fallback loading from environment variables.
6. `test_evidence_config_oversized_file_error`:
   - Rejects config file exceeding 64 KiB (`MAX_CONFIG_BYTES`).
7. `test_evidence_config_malformed_json_error`:
   - Handles corrupted JSON syntax gracefully.
8. `test_real_repo_evidence_config_file`:
   - Asserts repo-level `config/evidence.config.json` is present, well-formed, and valid.

## 3. Test Execution Output
```text
running 8 tests
test evidence_config::tests::test_evidence_config_default_is_valid ... ok
test evidence_config::tests::test_evidence_config_from_env_fallback ... ok
test evidence_config::tests::test_evidence_config_from_path_and_missing ... ok
test evidence_config::tests::test_evidence_config_malformed_json_error ... ok
test evidence_config::tests::test_evidence_config_roundtrip_happy ... ok
test evidence_config::tests::test_evidence_config_oversized_file_error ... ok
test evidence_config::tests::test_evidence_config_validation_failures ... ok
test evidence_config::tests::test_real_repo_evidence_config_file ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 143 filtered out; finished in 0.02s
```
