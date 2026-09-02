# T-00455 — Documentation Index Control / configuration: Unit Test

## 1. Unit Test Scope
This task tests `DocIndexConfig` across valid, invalid, boundary, and fallback conditions.

## 2. Test Cases & Coverage
1. `test_doc_index_config_default_is_valid`: Asserts default config validity and standard fields.
2. `test_doc_index_config_roundtrip_happy`: Validates JSON serialization and deserialization symmetry.
3. `test_doc_index_config_from_path_and_missing`: Tests successful file loading and non-existent file error handling.
4. `test_doc_index_config_validation_failures`: Tests strict negative cases:
   - Empty `version`
   - Empty `root_dirs`
   - Path traversal components (`..`) in `root_dirs`
   - Empty `include_extensions`
   - Extensions without leading dot (`.`)
   - Malformed JSON string
5. `test_doc_index_config_from_env_fallback`: Validates fallback to defaults when environment variable is absent.

## 3. Test Execution Output
```text
running 5 tests
test doc_index_config::tests::test_doc_index_config_default_is_valid ... ok
test doc_index_config::tests::test_doc_index_config_from_env_fallback ... ok
test doc_index_config::tests::test_doc_index_config_roundtrip_happy ... ok
test doc_index_config::tests::test_doc_index_config_from_path_and_missing ... ok
test doc_index_config::tests::test_doc_index_config_validation_failures ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 114 filtered out; finished in 0.01s
```
