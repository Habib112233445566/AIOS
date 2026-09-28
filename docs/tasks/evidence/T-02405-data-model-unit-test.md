# T-02405: Sandbox Enforcement Data Model Unit Test

## 1. Test Suite Summary
Created and executed focused automated unit tests in `code/aiosh-rust/aiosh-core/tests/test_sandbox_data_model.rs` validating the entire data model under both positive and negative constraints.

## 2. Test Cases Covered
1. `test_valid_profiles_default_construction`:
   - Validates `standard()`, `strict()`, and `permissive()` profiles against isolation levels, profile types, network modes, and environment policies.
2. `test_resource_limits_boundary_conditions`:
   - Tests boundary values for memory (1MB to 64GB), wall-clock execution limits (10ms to 1h), process counts (1 to 4096), and open file descriptors (16 to 65536).
   - Confirms negative validation rejection when limits drop below minima or exceed maxima.
3. `test_filesystem_policy_traversal_and_empty`:
   - Validates rejection of empty path entries.
   - Enforces strict rejection of directory traversal sequences (`..`) in read-only, read-write, and denied path sets.
4. `test_filesystem_policy_conflicts`:
   - Confirms rejection of policy definitions that contain mutually exclusive permissions (e.g., path declared both read-only and read-write, or read-write and denied).
5. `test_custom_profile_builder_workflow`:
   - Exercises `SandboxProfileBuilder` creating custom, verified profile with customized isolation level, resource ceilings, paths, network egress modes, syscall denylists, and injected environment variables.
6. `test_execution_request_and_result`:
   - Enforces rejection of empty command strings and traversal working directories (`cwd`).
   - Asserts execution request validity with PEP grant and session identifiers.
   - Validates `SandboxExecutionResult` success status checks.
7. `test_json_roundtrip_and_canonical_hash`:
   - Confirms deterministic serde JSON serialization/deserialization and byte-identical canonical SHA-256 hash calculation.

## 3. Results
All 7 unit and integration tests passed with 0 failures and 0 compiler warnings.
