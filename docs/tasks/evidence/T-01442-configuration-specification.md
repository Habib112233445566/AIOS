# T-01442: User Session Bootstrap — Configuration: Specification

## Metadata
- **Task ID:** `T-01442`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** User Session Bootstrap Configuration Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Specification Overview & System Contract

This specification formalizes the API, data structures, invariants, error taxonomy, and resolution precedence for the AIOS User Session Bootstrap Configuration Subsystem (`code/aiosh-rust/aiosh-core/src/session_config.rs`).

The configuration subsystem provides deterministic, type-safe configuration resolution for operator CLI and autonomous agent MCP session workflows, replacing ad-hoc parameter passing with centralized governance.

---

## 2. Invariants Taxonomy (`SC1..SC7`)

The configuration subsystem strictly enforces invariants `SC1..SC7`:

| Invariant | Name | Rule & Bound |
|---|---|---|
| **SC1** | Store Path Validity | `store_path` must be non-empty, $\le 1024$ bytes, and contain zero ASCII control characters or null bytes (`\0`). |
| **SC2** | User Session Capacity Bounds | `max_sessions_per_user` must be an integer within $[1 \dots 128]$ (default: `32`). |
| **SC3** | Total Store Capacity Bounds | `max_total_sessions` must be an integer within $[10 \dots 10,000]$ (default: `1,024`). |
| **SC4** | Idle Timeout Bounds | `default_idle_timeout_seconds` must be an integer within $[10 \dots 86,400]$ seconds (default: `900`s = 15m). |
| **SC5** | Store Sizing Bounds | `max_store_size_bytes` must be within $[65,536 \dots 104,857,600]$ (64 KiB .. 100 MiB; default: `10,485,760` = 10 MiB). |
| **SC6** | Resolution Precedence | Hierarchical resolution: Explicit File > Environment Variables > Built-in Defaults. |
| **SC7** | File Read Bounds & Fail-Loud | On-disk configuration files must be valid UTF-8, bounded to $\le 65,536$ bytes (64 KiB), and must reject invalid schema or values loudly. |

---

## 3. Data Model (`SessionConfig`)

### Rust Type Definition
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Filesystem path to the JSON session store.
    pub store_path: PathBuf,
    /// Maximum concurrent active sessions per individual user account (1..128).
    pub max_sessions_per_user: usize,
    /// Maximum total sessions stored across all users (10..10,000).
    pub max_total_sessions: usize,
    /// Default inactivity threshold in seconds before auto-locking (10..86,400).
    pub default_idle_timeout_seconds: u64,
    /// Maximum allowed session store file size on disk in bytes (64 KiB..100 MiB).
    pub max_store_size_bytes: u64,
    /// Whether mutations automatically commit to disk without explicit flags.
    pub auto_persist: bool,
}
```

### Default Values
- `store_path`: `.aios/session_store.json`
- `max_sessions_per_user`: `32`
- `max_total_sessions`: `1024`
- `default_idle_timeout_seconds`: `900`
- `max_store_size_bytes`: `10 * 1024 * 1024` (10 MiB)
- `auto_persist`: `true`

---

## 4. Environment Variables Mapping (`SC6`)

| Environment Variable | Target Field | Valid Range | Parsing Error Behavior |
|---|---|---|---|
| `AIOS_SESSION_STORE_PATH` | `store_path` | $1 \le \text{len} \le 1024$ chars, no control chars | Reverts or errors loudly |
| `AIOS_SESSION_MAX_PER_USER` | `max_sessions_per_user` | $[1 \dots 128]$ | Returns explicit parse error |
| `AIOS_SESSION_MAX_TOTAL` | `max_total_sessions` | $[10 \dots 10,000]$ | Returns explicit parse error |
| `AIOS_SESSION_IDLE_TIMEOUT_SECS` | `default_idle_timeout_seconds` | $[10 \dots 86,400]$ | Returns explicit parse error |
| `AIOS_SESSION_MAX_STORE_SIZE_BYTES` | `max_store_size_bytes` | $[65,536 \dots 104,857,600]$ | Returns explicit parse error |
| `AIOS_SESSION_AUTO_PERSIST` | `auto_persist` | `true`, `false`, `1`, `0` | Returns explicit parse error |

---

## 5. Resolution API Specification

```rust
impl SessionConfig {
    /// Validates current struct against invariants SC1..SC5.
    pub fn validate(&self) -> Result<(), String>;

    /// Reads and validates configuration from an explicit JSON file (SC7).
    pub fn from_file(path: &Path) -> Result<Self, String>;

    /// Reads environment variables and merges over default values (SC6).
    pub fn from_env() -> Result<Self, String>;

    /// Resolves configuration according to precedence: File > Env > Defaults (SC6).
    pub fn resolve(explicit_file: Option<&Path>) -> Result<Self, String>;
}
```

---

## 6. Error Taxonomy

All configuration operations return deterministic, non-empty error descriptions formatted as:
`SC<N> violation: <description>`

Examples:
- `SC1 violation: store_path cannot be empty`
- `SC1 violation: store_path length (1050 bytes) exceeds maximum limit of 1024 bytes`
- `SC2 violation: max_sessions_per_user must be between 1 and 128, got 200`
- `SC3 violation: max_total_sessions must be between 10 and 10000, got 5`
- `SC4 violation: default_idle_timeout_seconds must be between 10 and 86400, got 2`
- `SC5 violation: max_store_size_bytes must be between 65536 (64 KiB) and 104857600 (100 MiB), got 1000`
- `SC7 violation: session config at '...' exceeds maximum allowed size of 64 KiB`
