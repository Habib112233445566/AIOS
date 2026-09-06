# T-01343: Init & Service Supervision - Configuration: Scaffold

## Metadata
- **Task ID:** `T-01343`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Init & Service Supervision Configuration Subsystem Skeleton (`aiosh-core::service_config`)
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Scaffold Overview

This task creates the module skeleton and typed interfaces for the Init & Service Supervision Configuration Subsystem (`aiosh-core::service_config`). In accordance with scaffold guidelines, functions and methods are defined with typed signatures and fail loudly (`unimplemented!`) until implemented in task `T-01344`.

---

## 2. Implemented Skeleton & Interfaces

Created file: `code/aiosh-rust/aiosh-core/src/service_config.rs`
Registered in: `code/aiosh-rust/aiosh-core/src/lib.rs`

### Data Structures & Constants:
- `DEFAULT_SERVICE_STORE_PATH`: `".aios/service_store.json"`
- `DEFAULT_TIMEOUT_START_SECS`: `30`
- `DEFAULT_TIMEOUT_STOP_SECS`: `30`
- `DEFAULT_MAX_STORE_SIZE_BYTES`: `10 * 1024 * 1024` (10 MiB)
- `DEFAULT_MAX_ENTITY_COUNT`: `10,000`
- `DEFAULT_AUTO_PERSIST`: `true`
- `DEFAULT_RESTART_BACKOFF_SECS`: `5`
- `DEFAULT_MAX_RESTART_BURST`: `5`
- `MAX_CONFIG_FILE_BYTES`: `65,536` (64 KiB)
- Invariant boundary constants (`MIN_TIMEOUT_SECS`, `MAX_TIMEOUT_SECS`, `MIN_STORE_SIZE_BYTES`, `MAX_ALLOWED_STORE_SIZE_BYTES`, `MIN_ENTITY_COUNT`, `MAX_ALLOWED_ENTITY_COUNT`, `MIN_RESTART_BACKOFF_SECS`, `MAX_RESTART_BACKOFF_SECS`, `MIN_RESTART_BURST`, `MAX_RESTART_BURST`).

### Type Definition:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub store_path: PathBuf,
    pub default_timeout_start_secs: u32,
    pub default_timeout_stop_secs: u32,
    pub max_store_size_bytes: u64,
    pub max_entity_count: usize,
    pub auto_persist: bool,
    pub restart_backoff_secs: u32,
    pub max_restart_burst: u32,
}
```

### Signatures Stubbed (fail loudly with unimplemented!):
- `impl Default for ServiceConfig`
- `pub fn validate(&self) -> Result<(), String>`
- `pub fn from_file(_path: &Path) -> Result<Self, String>`
- `pub fn from_env() -> Result<Self, String>`
- `pub fn resolve(_config_path_opt: Option<&Path>) -> Result<Self, String>`

---

## 3. Verification & Acceptance Criteria

- [x] `code/aiosh-rust/aiosh-core/src/service_config.rs` created with typed signatures.
- [x] Exported in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- [x] Bodies fail loudly via `unimplemented!`.
- [x] Project compiles and unit test stubs pass.
