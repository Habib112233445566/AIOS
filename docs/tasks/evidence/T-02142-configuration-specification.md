# Task Evidence: T-02142 (PEP Decision Engine Configuration: Specification)

## Overview
- **Task ID**: `T-02142`
- **Task Name**: configuration: Specification
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Timestamp**: 2026-09-21T00:55:30+05:00
- **Status**: COMPLETED

## Technical Specification: `PepConfig`

### 1. Data Structure Schema
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PepConfig {
    /// Configuration schema version (default: "1.0.0").
    pub version: String,
    /// Path to persistent policy store JSON file.
    pub store_path: PathBuf,
    /// Maximum allowed file size for the policy store (1 KiB .. 100 MiB).
    pub max_store_bytes: u64,
    /// Maximum rule capacity in memory (1 .. 50,000).
    pub max_rules: usize,
    /// Default combining algorithm for rule evaluation.
    pub default_algorithm: PepCombiningAlgorithm,
    /// Whether every evaluation generates an audit row.
    pub audit_all_evaluations: bool,
    /// Whether unparseable store files are automatically quarantined.
    pub auto_quarantine_corrupt: bool,
}
```

### 2. Constants & Boundaries
- `MAX_CONFIG_BYTES = 65,536` (64 KiB max configuration file size).
- `DEFAULT_PEP_STORE_PATH = ".aios/pep_policies.json"`.
- `MIN_STORE_BYTES = 1,024` (1 KiB).
- `MAX_STORE_BYTES = 104,857,600` (100 MiB).
- `DEFAULT_MAX_STORE_BYTES = 10,485,760` (10 MiB).
- `MIN_RULES_COUNT = 1`.
- `MAX_RULES_COUNT = 50,000`.
- `DEFAULT_MAX_RULES = 5,000`.

### 3. Error Codes
- `PEPCONF_ERR_IO`: Filesystem IO error during load or save.
- `PEPCONF_ERR_PARSE`: Malformed JSON or syntax failure.
- `PEPCONF_ERR_VALIDATION`: Path traversal, control characters, or invalid extension.
- `PEPCONF_ERR_BOUNDS`: Numerical boundary breach (`max_rules` or `max_store_bytes`).

### 4. Method Contracts
1. `validate(&self) -> Result<(), String>`:
   - Validates `store_path` using path hygiene rules (no `..`, no control characters, length $\le 1024$, `.json` extension).
   - Validates `max_store_bytes` $\in [1\,024, 104\,857\,600]$.
   - Validates `max_rules` $\in [1, 50\,000]$.
   - Validates `version` is non-empty.
2. `from_json(json_str: &str) -> Result<Self, String>`: Parses JSON and executes `validate()`.
3. `to_json(&self) -> Result<String, String>`: Validates and serializes pretty-printed JSON.
4. `from_path(path: &Path) -> Result<Self, String>`: Reads up to `MAX_CONFIG_BYTES`, rejects symlinks, and deserializes.
5. `save_to_path(&self, path: &Path) -> Result<(), String>`: Validates and atomically writes via `.tmp.<pid>` rename.
6. `from_env() -> Result<Self, String>`:
   - Checks `AIOSH_PEP_CONFIG` for config file path.
   - Overrides `store_path` with `AIOSH_PEP_STORE_PATH`.
   - Overrides `max_rules` with `AIOSH_PEP_MAX_RULES`.
   - Overrides `default_algorithm` with `AIOSH_PEP_DEFAULT_ALGORITHM`.
