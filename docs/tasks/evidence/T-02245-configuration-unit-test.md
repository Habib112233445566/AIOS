# Task Evidence: T-02245 (Grant Lifecycle Configuration: Unit Test)

## Overview
- **Task ID**: `T-02245`
- **Task Name**: Grant Lifecycle Configuration: Unit Test
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Grant Lifecycle
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Timestamp**: 2026-09-23T01:18:00+05:00
- **Status**: COMPLETED

## Unit Test Suite Results

Test file: `code/aiosh-rust/aiosh-core/tests/test_pep_grant_config.rs`
Command: `cargo test -p aiosh-core --test test_pep_grant_config`

### Test Execution Output
```text
running 8 tests
test test_pep_grant_config_default ... ok
test test_pep_grant_config_from_env ... ok
test test_pep_grant_config_json_roundtrip ... ok
test test_pep_grant_config_validation_bounds ... ok
test test_pep_grant_config_validation_empty_version ... ok
test test_pep_grant_config_validation_invalid_extension ... ok
test test_pep_grant_config_validation_path_traversal ... ok
test test_pep_grant_config_file_persistence_roundtrip ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

### Coverage of Invariants (`GRANTCONF1..GRANTCONF6`)
- `test_pep_grant_config_default`: Asserts default values, version `1.0.0`, store path `.aios/pep_grants.json`, max store bytes 10 MiB, max grants 5,000, delegation depth 3, auto sweep true, cascade revocation false.
- `test_pep_grant_config_json_roundtrip`: Verifies roundtrip serialization and deserialization fidelity.
- `test_pep_grant_config_validation_empty_version`: Enforces non-empty schema version (`GRANTCONF_ERR_VALIDATION`).
- `test_pep_grant_config_validation_path_traversal`: Enforces rejection of parent directory traversal (`..`) (`GRANTCONF_ERR_VALIDATION`).
- `test_pep_grant_config_validation_invalid_extension`: Enforces `.json` extension requirement (`GRANTCONF_ERR_VALIDATION`).
- `test_pep_grant_config_validation_bounds`: Enforces bounds checking on `max_grants` ($[1, 50\,000]$), `max_store_bytes` ($[1\,024, 104\,857\,600]$), and `default_max_delegation_depth` ($[1, 10]$) with `GRANTCONF_ERR_BOUNDS`.
- `test_pep_grant_config_file_persistence_roundtrip`: Validates atomic saving and loading from disk.
- `test_pep_grant_config_from_env`: Validates complete parsing and overriding from environment variables.
