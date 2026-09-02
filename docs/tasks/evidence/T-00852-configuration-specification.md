# T-00852 — Regression Triage / Configuration: Specification

## 1. Data Contract & Invariants

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriageConfig {
    pub max_store_bytes: usize,
    pub default_severity: TriageSeverity,
    pub auto_ingest_suites: Vec<String>,
    pub retention_days: u32,
    pub notify_blockers: bool,
    pub store_path: Option<String>,
}
```

### Constraints & Validation Rules:
1. `max_store_bytes`: Must be between `16_384` (16 KiB) and `67_108_864` (64 MiB). Default: `1_048_576` (1 MiB).
2. `retention_days`: Must be $\ge 1$. Default: `90`.
3. `auto_ingest_suites`: Non-empty list of string filters (default: `["*"]`).
4. `config_file_size_cap`: Configuration files must not exceed 64 KiB (`MAX_CONFIG_FILE_BYTES = 65536`).

## 2. API Methods
- `TriageConfig::default() -> Self`
- `TriageConfig::validate(&self) -> Result<(), String>`
- `TriageConfig::from_file(path: &Path) -> Result<Self, String>`
- `TriageConfig::from_env_or_default() -> Self`
- `TriageConfig::save_to_file(&self, path: &Path) -> Result<(), String>`
