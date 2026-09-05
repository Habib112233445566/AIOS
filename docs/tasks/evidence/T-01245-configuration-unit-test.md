# T-01245: Package Management - Configuration: Unit Test

## Metadata
- **Task ID:** `T-01245`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Package Management Configuration Unit Test Suite
- **Status:** Complete

## 1. Test Suite Architecture
Authored a dedicated unit test suite in `code/aiosh-rust/aiosh-core/tests/test_package_config.rs` exercising `PackageConfig` against criteria PC1..PC6.

### Test Matrix:
1. `test_package_config_defaults_and_validation`:
   - Validates that canonical defaults adhere to `PC1..PC6`.
   - Checks default field values (`.aios/packages.json`, `deb`, 10 MiB, 10,000 count, 2 allowed repos).
2. `test_package_config_pc1_store_path_invariants`:
   - Rejects empty path strings (`""`).
   - Rejects paths exceeding 1,024 bytes (`1025` bytes).
   - Validates boundary path length of exactly `1024` bytes.
   - Rejects paths containing newline control characters (`\n`).
   - Rejects paths containing null bytes (`\0`).
3. `test_package_config_pc2_store_size_invariants`:
   - Rejects size below minimum (<64 KiB).
   - Validates exact minimum boundary (64 KiB / 65,536 bytes).
   - Validates exact maximum boundary (100 MiB / 104,857,600 bytes).
   - Rejects size exceeding maximum (>100 MiB).
4. `test_package_config_pc3_entity_count_invariants`:
   - Rejects count below minimum (<10).
   - Validates exact minimum boundary (10).
   - Validates exact maximum boundary (100,000).
   - Rejects count exceeding maximum (>100,000).
5. `test_package_config_pc4_repository_security`:
   - Rejects insecure plaintext `http://` upstream repositories.
   - Rejects non-HTTP protocols (e.g. `ftp://`).
   - Validates secure `https://` upstream repositories.
   - Validates local `file://` repository mirrors.
   - Rejects control characters in repository URLs.
6. `test_package_config_pc5_env_resolution`:
   - Sets environment variables (`AIOS_PACKAGE_STORE_PATH`, `AIOS_PACKAGE_DEFAULT_FORMAT`, `AIOS_PACKAGE_MAX_STORE_SIZE_BYTES`, `AIOS_PACKAGE_MAX_ENTITIES`, `AIOS_PACKAGE_AUTO_PERSIST`, `AIOS_PACKAGE_ALLOWED_REPOS`).
   - Verifies `PackageConfig::from_env()` accurately parses and applies all environment settings.
7. `test_package_config_pc6_file_roundtrip_and_size_cap`:
   - Tests file persistence, serialization, and deserialization via `from_file`.
   - Verifies precedence via `PackageConfig::resolve(Some(&path))`.
   - Rejects configuration files exceeding 64 KiB ceiling with explicit `PC6 violation`.

## 2. Test Execution Output
Executed in isolation via `cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_package_config`:
```
running 7 tests
test test_package_config_pc2_store_size_invariants ... ok
test test_package_config_pc3_entity_count_invariants ... ok
test test_package_config_defaults_and_validation ... ok
test test_package_config_pc1_store_path_invariants ... ok
test test_package_config_pc4_repository_security ... ok
test test_package_config_pc5_env_resolution ... ok
test test_package_config_pc6_file_roundtrip_and_size_cap ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```
