# T-00452 — Documentation Index Control / configuration: Specification

## 1. Specification Overview
This document specifies the data model, validation rules, resolution hierarchy, and serialization contracts for `DocIndexConfig` in `code/aiosh-rust/aiosh-core/src/doc_index_config.rs`.

## 2. Configuration Schema

### Rust Struct Definition:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocIndexConfig {
    pub version: String,
    pub root_dirs: Vec<String>,
    pub include_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub enforce_strict_links: bool,
}
```

### JSON Schema:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DocIndexConfig",
  "type": "object",
  "required": ["version", "root_dirs", "include_extensions", "exclude_patterns", "enforce_strict_links"],
  "properties": {
    "version": { "type": "string" },
    "root_dirs": {
      "type": "array",
      "items": { "type": "string" },
      "minItems": 1,
      "maxItems": 50
    },
    "include_extensions": {
      "type": "array",
      "items": { "type": "string" },
      "minItems": 1
    },
    "exclude_patterns": {
      "type": "array",
      "items": { "type": "string" }
    },
    "enforce_strict_links": { "type": "boolean" }
  }
}
```

## 3. Operations & Resolution Hierarchy

1. **`DocIndexConfig::default() -> Self`**:
   - `version`: `"1.0.0"`
   - `root_dirs`: `["docs"]`
   - `include_extensions`: `[".md"]`
   - `exclude_patterns`: `["**/node_modules/**", "**/target/**"]`
   - `enforce_strict_links`: `true`

2. **Resolution Hierarchy (`DocIndexConfig::from_env()`):**
   1. If `AIOS_DOC_INDEX_CONFIG` environment variable is set -> Load from specified path.
   2. Else if default file `docs/doc_index_config.json` exists -> Load from file.
   3. Otherwise -> Return `DocIndexConfig::default()`.

3. **Validation Invariants (`validate(&self)`):**
   - Non-empty `version`.
   - `root_dirs`: non-empty, max 50 items, no path traversal `../` elements.
   - `include_extensions`: non-empty, all elements start with `.` (e.g. `".md"`).
   - Read size cap of 64 KiB (`MAX_CONFIG_BYTES`).

## 4. PEP & Audit Policy
- Configuration resolution is read-only and safe for operator/agent consumption.
