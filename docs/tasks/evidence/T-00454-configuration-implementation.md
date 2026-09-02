# T-00454 — Documentation Index Control / configuration: Implementation

## 1. Implementation Scope
This task implements `DocIndexConfig` and configuration resolution methods in `code/aiosh-rust/aiosh-core/src/doc_index_config.rs`.

## 2. Implementation Details
- **`DocIndexConfig::default()`**: Provides baseline configuration indexing `"docs"` with `".md"` extensions.
- **`DocIndexConfig::from_json(json_str)` & `to_json()`**: Validated serialization with formatted JSON output.
- **`DocIndexConfig::from_path(path)`**: Bounded 64 KiB configuration file loader.
- **`DocIndexConfig::from_env()`**: Tiered fallback checking `AIOS_DOC_INDEX_CONFIG` -> `docs/doc_index_config.json` -> `default()`.
- **`validate(&self)`**: Enforces directory path non-emptiness, maximum 50 root dirs, path traversal prevention, and extension prefix format (`.`).

## 3. Test Verification
```text
running 3 tests
test doc_index_config::tests::test_doc_index_config_default_is_valid ... ok
test doc_index_config::tests::test_doc_index_config_roundtrip_happy ... ok
test doc_index_config::tests::test_doc_index_config_from_path_and_missing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 114 filtered out; finished in 0.22s
```
