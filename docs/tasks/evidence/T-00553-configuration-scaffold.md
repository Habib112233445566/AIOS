# T-00553 — Evidence & Audit Trail / configuration: Scaffold

## 1. Scaffold Scope
This task creates the configuration module skeleton `code/aiosh-rust/aiosh-core/src/evidence_config.rs` and registers the module export in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## 2. Scaffold Contents
- Defined `EvidenceConfig` struct with fields:
  - `evidence_dir: String`
  - `max_file_bytes: u64`
  - `allowed_extensions: Vec<String>`
  - `enforce_checksum: bool`
  - `require_all_steps: bool`
- Implemented `Default`, `validate`, `from_json`, `to_json`, `from_path`, and `from_env`.
- Exported `pub mod evidence_config;` in `lib.rs`.

## 3. Test Verification
```text
running 1 test
test evidence_config::tests::test_evidence_config_default_is_valid ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 143 filtered out; finished in 0.00s
```
