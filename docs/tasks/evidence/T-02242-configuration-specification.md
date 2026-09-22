# Task Evidence: T-02242 (Grant Lifecycle Configuration: Specification)

## Overview
- **Task ID**: `T-02242`
- **Task Name**: Grant Lifecycle Configuration: Specification
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Grant Lifecycle
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Timestamp**: 2026-09-23T01:09:00+05:00
- **Status**: COMPLETED

## Technical Specification: `PepGrantConfig`

### 1. Data Structure Schema
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PepGrantConfig {
    /// Configuration schema version (default: "1.0.0").
    pub version: String,
    /// Path to persistent grant store JSON file.
    pub store_path: PathBuf,
    /// Maximum allowed file size for the grant store (1 KiB .. 100 MiB).
    pub max_store_bytes: u64,
    /// Maximum grant capacity in memory (1 .. 50,000).
    pub max_grants: usize,
    /// Default maximum delegation depth for newly issued grants (1 .. 10).
    pub default_max_delegation_depth: u32,
    /// Whether expired grants are automatically swept upon service load.
    pub auto_sweep_on_load: bool,
    /// Whether grant revocation cascades to derived child grants by default.
    pub cascade_revocation_by_default: bool,
}
```

### 2. Constants & Boundaries
- `MAX_CONFIG_BYTES: u64 = 64 * 1024` (64 KiB max configuration file size).
- `DEFAULT_PEP_GRANT_STORE_PATH: &str = ".aios/pep_grants.json"`.
- `MIN_STORE_BYTES: u64 = 1,024` (1 KiB).
- `MAX_STORE_BYTES: u64 = 104_857_600` (100 MiB).
- `DEFAULT_MAX_STORE_BYTES: u64 = 10_485_760` (10 MiB).
- `MIN_GRANTS_COUNT: usize = 1`.
- `MAX_GRANTS_COUNT: usize = 50,000`.
- `DEFAULT_MAX_GRANTS: usize = 5,000`.
- `MIN_DELEGATION_DEPTH: u32 = 1`.
- `MAX_DELEGATION_DEPTH: u32 = 10`.
- `DEFAULT_MAX_DELEGATION_DEPTH: u32 = 3`.

### 3. Error Codes
- `GRANTCONF_ERR_IO`: Filesystem IO error during load or save.
- `GRANTCONF_ERR_PARSE`: Malformed JSON or syntax failure.
- `GRANTCONF_ERR_VALIDATION`: Path traversal, control characters, or invalid extension.
- `GRANTCONF_ERR_BOUNDS`: Numerical boundary breach (`max_grants`, `max_store_bytes`, or `default_max_delegation_depth`).

### 4. Method Contracts
1. `validate(&self) -> Result<(), String>`:
   - Validates `store_path` using path hygiene rules (no `..`, no control characters, length $\le 1024$, `.json` extension).
   - Validates `max_store_bytes` $\in [1\,024, 104\,857\,600]$.
   - Validates `max_grants` $\in [1, 50\,000]$.
   - Validates `default_max_delegation_depth` $\in [1, 10]$.
   - Validates `version` is non-empty.
2. `from_json(json_str: &str) -> Result<Self, String>`:
   - Parses JSON string into `PepGrantConfig` and executes `validate()`.
3. `to_json(&self) -> Result<String, String>`:
   - Validates configuration and serializes to pretty-printed JSON.
4. `from_path(path: &Path) -> Result<Self, String>`:
   - Rejects symlinks, bounds file read to `MAX_CONFIG_BYTES`, and deserializes via `from_json`.
5. `save_to_path(&self, path: &Path) -> Result<(), String>`:
   - Validates configuration, creates parent directories if needed, writes to `.tmp.<pid>` temporary file, and atomically renames.
6. `from_env() -> Result<Self, String>`:
   - If `AIOSH_PEP_GRANT_CONFIG` is set, loads configuration from that path.
   - Applies environment variable overrides:
     - `AIOSH_PEP_GRANT_STORE_PATH` / `AIOSH_PEP_GRANT_STORE` -> `store_path`
     - `AIOSH_PEP_GRANT_MAX_GRANTS` -> `max_grants`
     - `AIOSH_PEP_GRANT_MAX_STORE_BYTES` -> `max_store_bytes`
     - `AIOSH_PEP_GRANT_MAX_DELEGATION_DEPTH` -> `default_max_delegation_depth`
     - `AIOSH_PEP_GRANT_AUTO_SWEEP` -> `auto_sweep_on_load`
     - `AIOSH_PEP_GRANT_CASCADE_REVOCATION` -> `cascade_revocation_by_default`
   - Validates resulting configuration before returning.

## Acceptance
- Full schema and contract detailed.
- Ready for scaffold implementation in `T-02243`.
