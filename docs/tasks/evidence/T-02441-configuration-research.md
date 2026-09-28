# T-02441: Sandbox Enforcement Configuration Research

## 1. Research Objectives
Establish authoritative facts, constraints, and architecture for the Configuration Subsystem of Sandbox Enforcement (`sandbox_config.rs`).

## 2. Existing Substrate & Prior Art
1. **Existing Minimal Config in `sandbox_service.rs`**:
   - `SandboxConfig` currently contains basic fields: `default_profile_name`, `max_output_capture_bytes`, `enforce_pep_grants`, `audit_enabled`.
2. **Standard AIOS Subsystem Config Patterns** (`audit_chain_config.rs`, `capability_config.rs`, `pep_config.rs`):
   - Strict versioning (`version: "1.0.0"`).
   - Dedicated constants for defaults and upper/lower bounds.
   - Structured error codes (`SANDBOXCONF_ERR_IO`, `SANDBOXCONF_ERR_PARSE`, `SANDBOXCONF_ERR_VALIDATION`, `SANDBOXCONF_ERR_BOUNDS`).
   - Atomic file I/O with maximum file size checks (`MAX_CONFIG_FILE_BYTES = 64 KiB`).
   - Environment variable overrides (`AIOS_SANDBOX_*`).
   - Clean serde serialization to/from JSON.

## 3. Facts vs Assumptions
- **Fact**: Configuration files must not exceed 64 KiB to prevent DoS via unbounded disk reading.
- **Fact**: Config validation must fail closed on unknown default profile names or out-of-bounds timeouts.
- **Assumption**: Operators and agents may specify a directory path where custom JSON profiles can be automatically discovered and loaded.

## 4. Key Design Decisions for Specification (T-02442)
1. Create `code/aiosh-rust/aiosh-core/src/sandbox_config.rs` implementing `SandboxConfig`.
2. Define boundary limits:
   - Output capture: 1 KiB to 64 MiB (default: 10 MiB).
   - Execution timeout: 1s to 86,400s (default: 300s).
   - Max registered profiles: 1 to 1,024 (default: 256).
3. Support environment variable overrides:
   - `AIOS_SANDBOX_DEFAULT_PROFILE`
   - `AIOS_SANDBOX_ENFORCE_PEP`
   - `AIOS_SANDBOX_MAX_OUTPUT_BYTES`
   - `AIOS_SANDBOX_TIMEOUT_SECS`
