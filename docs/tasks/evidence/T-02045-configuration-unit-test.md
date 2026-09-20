# Task Evidence: T-02045 (Capability Model / configuration: Unit Test)

## Task Information
- **Task ID**: T-02045
- **Title**: Capability Model / configuration: Unit Test
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Unit Tests
1. **Test Suite Location**: `code/aiosh-rust/aiosh-core/tests/test_capability_config.rs`.
2. **Test Cases Executed**:
   - `test_capability_config_defaults`:
     - Verifies default fields, getters, and default validity.
   - `test_capability_config_json_serde`:
     - Verifies round-trip JSON serialization/deserialization.
     - Tests custom JSON parsing.
   - `test_capability_config_validation_rules`:
     - Rejects empty version and versions > 32 characters.
     - Rejects empty store path, path traversal (`../`), null/control characters, and paths > 1024 characters.
     - Rejects store size out-of-bounds (< 1024 B or > 100 MB).
     - Rejects max capabilities out-of-bounds (0 or > 1,000,000).
     - Rejects expiration out-of-bounds (0 or > 10 years).
   - `test_capability_config_file_and_env`:
     - Tests file loading from disk via `from_path`.
     - Tests environment variable overrides: `AIOS_CAPABILITY_STORE_PATH`, `AIOS_CAPABILITY_MAX_CAPABILITIES`, `AIOS_CAPABILITY_MAX_STORE_BYTES`.
   - `test_capability_service_from_config_and_capacity_enforcement`:
     - Verifies `CapabilityService::from_config`.
     - Verifies strict enforcement of configured `max_capabilities` on both root issuance and attenuation.

3. **Results**:
   - All 5 unit tests compiled and passed with 0 failures.
