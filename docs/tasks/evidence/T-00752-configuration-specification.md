# T-00752 — Secrets & Access Hygiene / configuration: Specification

## 1. Data Contract (`SecretsConfig`)
The configuration contract defines runtime operational tuning for secret scanners:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretsConfig {
    pub version: String,
    pub max_file_bytes: u64,
    pub max_line_bytes: usize,
    pub ignored_dirs: Vec<String>,
    pub allow_patterns: Vec<String>,
    pub require_clean: bool,
}
```

## 2. Default Configuration Values
```json
{
  "version": "1.0.0",
  "max_file_bytes": 16777216,
  "max_line_bytes": 4096,
  "ignored_dirs": [
    ".git",
    "target",
    "node_modules",
    ".venv",
    "dist"
  ],
  "allow_patterns": [],
  "require_clean": false
}
```

## 3. Validation Bounds & Error Invariants
- `version`: Non-empty, $\le 32$ characters.
- `max_file_bytes`: Range $[1024, 1073741824]$ bytes ($1 \text{ KiB} .. 1 \text{ GiB}$).
- `max_line_bytes`: Range $[128, 65536]$ bytes.
- `ignored_dirs`: Between $1$ and $50$ directory names, each non-empty.
- `allow_patterns`: At most $100$ exemption patterns.

## 4. Resolution Order
1. Path provided explicitly via `SecretsConfig::from_path(path)`.
2. Path supplied in environment variable `AIOS_SECRETS_CONFIG`.
3. Default path on disk `docs/secrets_config.json`.
4. Fallback defaults via `SecretsConfig::default()`.
