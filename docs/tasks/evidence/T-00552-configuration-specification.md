# T-00552 — Evidence & Audit Trail / configuration: Specification

## 1. Specification Overview
This specification defines the schema, validation rules, environmental loading precedence, and error invariants for `EvidenceConfig` in `aiosh-core`.

## 2. Configuration Schema & Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceConfig {
    pub evidence_dir: String,
    pub max_file_bytes: u64,
    pub allowed_extensions: Vec<String>,
    pub enforce_checksum: bool,
    pub require_all_steps: bool,
}
```

### Defaults:
- `evidence_dir`: `"docs/tasks/evidence"`
- `max_file_bytes`: `16_777_216` (16 MiB)
- `allowed_extensions`: `[".md", ".json"]`
- `enforce_checksum`: `true`
- `require_all_steps`: `false`

## 3. Validation Rules
- **`evidence_dir`**: Must be a non-empty relative path without `..` traversal segments or `:` path prefixes.
- **`max_file_bytes`**: Must be strictly greater than 0 and less than or equal to `67_108_864` (64 MiB).
- **`allowed_extensions`**: Must contain at least one extension, and all entries must start with `.`.
- **Config file size**: Reading a config file from disk is strictly bounded by 64 KiB (`MAX_CONFIG_BYTES = 65_536`).

## 4. Precedence Hierarchy
1. `AIOS_EVIDENCE_CONFIG_PATH`: Path to explicit JSON config file.
2. Direct environment variables (`AIOS_EVIDENCE_DIR`, `AIOS_EVIDENCE_MAX_FILE_BYTES`).
3. Repository default: `<repo_root>/config/evidence.config.json`.
4. In-memory fallback (`EvidenceConfig::default()`).
