# T-00651 — Repository Health / configuration: Research

## 1. Context & Prior Art
Repository health diagnostics require configurable thresholds and boundary rules across different deployment environments. For example, large monorepos may customize file size caps, ignore special artifact directories, or require clean Git status in automated release pipelines.

## 2. Configuration Architecture & Invariants

### A. Configuration Schema (`RepoHealthConfig`)
- `version`: Semver identifier (default: `"1.0.0"`).
- `max_file_bytes`: Maximum allowed single file size (default: 16 MiB = `16,777,216` bytes).
- `ignored_dirs`: List of directory names excluded from recursive scanning (default: `[".git", "target", "node_modules", ".venv"]`).
- `require_clean_git`: Boolean controlling whether uncommitted git changes trigger `Fail` instead of `Warn` (default: `false`).
- `security_policy_path`: Relative path to security policy file (default: `"SECURITY.md"`).
- `min_security_policy_bytes`: Minimum byte length for security policy (default: `100`).

### B. Persistence & Resolution Order
1. Explicit path passed to parser (`RepoHealthConfig::from_path`).
2. Environment variable `AIOS_REPO_HEALTH_CONFIG`.
3. Default configuration file `docs/repo_health_config.json`.
4. In-memory default fallback (`RepoHealthConfig::default()`).

### C. Validation & Safety Invariants
- `MAX_CONFIG_BYTES`: Bounded to 64 KiB (`65,536` bytes) to prevent resource exhaustion.
- `ignored_dirs`: Capped at 50 entries to prevent algorithmic degradation.

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Working Assumption |
| :--- | :--- | :--- |
| **Config Patterns** | Subsystems in `aiosh-core` implement `from_json`, `to_json`, `from_path`, `from_env`, and `validate`. | `RepoHealthConfig` follows the identical configuration pattern. |
| **Bounded Reads** | Config file reading uses `.take(MAX_CONFIG_BYTES)`. | Reading up to 64 KiB prevents memory exhaustion on malformed paths. |
| **Fail-Closed** | Malformed JSON or invalid validation fails with descriptive error strings. | `Result<RepoHealthConfig, String>` preserves fail-closed error propagation. |

## 4. Key Design Decisions for Implementation
1. Create `code/aiosh-rust/aiosh-core/src/repo_health_config.rs`.
2. Register `pub mod repo_health_config;` in `lib.rs`.
3. Provide full serialization and deserialization test suite.
