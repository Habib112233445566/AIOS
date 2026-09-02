# T-00652 — Repository Health / configuration: Specification

## 1. Specification Overview
The `RepoHealthConfig` module provides strongly typed, validated configuration parameters for repository health check suites across AIOS.

## 2. Configuration Schema & Types

### 2.1 Struct Definition
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoHealthConfig {
    pub version: String,
    pub max_file_bytes: u64,
    pub ignored_dirs: Vec<String>,
    pub require_clean_git: bool,
    pub security_policy_path: String,
    pub min_security_policy_bytes: u64,
}
```

### 2.2 JSON Schema Example
```json
{
  "version": "1.0.0",
  "max_file_bytes": 16777216,
  "ignored_dirs": [
    ".git",
    "target",
    "node_modules",
    ".venv"
  ],
  "require_clean_git": false,
  "security_policy_path": "SECURITY.md",
  "min_security_policy_bytes": 100
}
```

## 3. Validation Rules
1. `version`: Non-empty string, trimmed length $\le 32$ characters.
2. `max_file_bytes`: Range $1,024 \le \text{max\_file\_bytes} \le 1,073,741,824$ (1 KiB to 1 GiB).
3. `ignored_dirs`: Array of $1 \le \text{len} \le 50$ directory names; elements non-empty and must not contain `..` or path separators `/`, `\`.
4. `security_policy_path`: Non-empty relative path $\le 255$ chars, must not contain `..`.
5. `min_security_policy_bytes`: Range $1 \le \text{bytes} \le 65,536$.

## 4. Error Conditions
- File size $> 64$ KiB (`MAX_CONFIG_BYTES`) returns an error during file reading.
- Malformed JSON returns `Err("Failed to parse RepoHealthConfig JSON: <err>")`.
- Constraint violations return descriptive error strings naming the failing field.
