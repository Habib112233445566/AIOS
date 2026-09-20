# Task Evidence: T-02042 (Capability Model / configuration: Specification)

## Task Information
- **Task ID**: T-02042
- **Title**: Capability Model / configuration: Specification
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Status**: Completed
- **Date**: 2026-09-20

## Capability Configuration Specification (`CapabilityConfig`)

### 1. Data Structure
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityConfig {
    pub version: String,
    pub store_path: PathBuf,
    pub max_store_bytes: u64,
    pub max_capabilities: usize,
    pub default_expires_secs: Option<u64>,
    pub enforce_strict_monotonic: bool,
    pub auto_prune_on_load: bool,
}
```

### 2. Field Specifications and Bounds
| Field | Type | Default | Validation Range / Constraints |
|---|---|---|---|
| `version` | `String` | `"1.0.0"` | Non-empty, $\le 32$ chars, ASCII only |
| `store_path` | `PathBuf` | `".aios/capability_store.json"` | Non-empty, $\le 1024$ chars, no control chars, no `..` traversal |
| `max_store_bytes` | `u64` | `10_485_760` (10 MiB) | $1\,024 \le x \le 104\,857\,600$ (1 KiB to 100 MiB) |
| `max_capabilities` | `usize` | `10_000` | $1 \le x \le 1\,000\,000$ |
| `default_expires_secs` | `Option<u64>` | `None` | If `Some(x)`: $1 \le x \le 315\,360\,000$ (1s to 10 yrs) |
| `enforce_strict_monotonic` | `bool` | `true` | Boolean flag |
| `auto_prune_on_load` | `bool` | `true` | Boolean flag |

### 3. Environment Variables
- `AIOS_CAPABILITY_CONFIG`: Direct path to a JSON configuration file.
- `AIOS_CAPABILITY_STORE_PATH`: Overrides `store_path`.
- `AIOS_CAPABILITY_MAX_CAPABILITIES`: Overrides `max_capabilities` (parsed as `usize`).
- `AIOS_CAPABILITY_MAX_STORE_BYTES`: Overrides `max_store_bytes` (parsed as `u64`).

### 4. Lifecycle & Methods
- `default() -> Self`: Returns default safe configuration.
- `from_json(json_str: &str) -> Result<Self, String>`: Parses JSON and validates.
- `to_json(&self) -> Result<String, String>`: Validates and serializes to pretty JSON.
- `from_path(path: &Path) -> Result<Self, String>`: Loads from file, capped at `MAX_CONFIG_BYTES` (64 KiB), parses and validates.
- `from_env() -> Result<Self, String>`: Loads from `AIOS_CAPABILITY_CONFIG` if set, else defaults with environment variable overrides applied, and validates.
- `validate(&self) -> Result<(), String>`: Enforces all bounds, security invariants, and path traversal protections.
