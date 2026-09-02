# T-00654 — Repository Health / configuration: Implementation

## 1. Implementation Scope
This task implements `RepoHealthConfig` in `code/aiosh-rust/aiosh-core/src/repo_health_config.rs`.

## 2. Implementation Deliverables
- Implemented `RepoHealthConfig`:
  - `Default` providing fallback values (`version: "1.0.0"`, `max_file_bytes: 16MiB`, `ignored_dirs: [".git", "target", "node_modules", ".venv"]`, `min_security_policy_bytes: 100`).
  - `from_json` & `to_json` with schema validation.
  - `from_path` with `MAX_CONFIG_BYTES` (64 KiB) bounding.
  - `from_env` supporting `AIOS_REPO_HEALTH_CONFIG` environment variable.
  - `validate` enforcing strict boundary invariants.
- Added 3 unit tests covering default values, JSON roundtrip, validation failure paths, and temporary path reading.

## 3. Test Verification Output
```text
running 3 tests
test repo_health_config::tests::test_repo_health_config_default_and_roundtrip ... ok
test repo_health_config::tests::test_repo_health_config_validation_errors ... ok
test repo_health_config::tests::test_repo_health_config_from_path ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 167 filtered out; finished in 0.05s
```
