# T-00554 — Evidence & Audit Trail / configuration: Implementation

## 1. Implementation Scope
This task implements the configuration data model, JSON serialization, validation invariants, filesystem loading, and environment variable resolution for Evidence & Audit Trail in `code/aiosh-rust/aiosh-core/src/evidence_config.rs`, and creates the repository default `config/evidence.config.json`.

## 2. Implementation Details
- `EvidenceConfig` struct:
  - `evidence_dir`: `"docs/tasks/evidence"`
  - `max_file_bytes`: `16_777_216` (16 MiB)
  - `allowed_extensions`: `[".md", ".json"]`
  - `enforce_checksum`: `true`
  - `require_all_steps`: `false`
- `EvidenceConfig::validate()`: Enforces relative paths, denies path traversal (`..`), checks size upper bound (64 MiB), and enforces extension prefix (`.`).
- `EvidenceConfig::from_path()`: Enforces 64 KiB config size limit to protect against unbounded config files.
- `EvidenceConfig::from_env()`: Resolves `AIOS_EVIDENCE_CONFIG_PATH`, `AIOS_EVIDENCE_DIR`, and `AIOS_EVIDENCE_MAX_FILE_BYTES`.
- Created `config/evidence.config.json`.

## 3. Test Verification
```text
running 5 tests
test evidence_config::tests::test_evidence_config_default_is_valid ... ok
test evidence_config::tests::test_evidence_config_from_env_fallback ... ok
test evidence_config::tests::test_evidence_config_roundtrip_happy ... ok
test evidence_config::tests::test_evidence_config_from_path_and_missing ... ok
test evidence_config::tests::test_evidence_config_validation_failures ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 143 filtered out; finished in 0.08s
```
