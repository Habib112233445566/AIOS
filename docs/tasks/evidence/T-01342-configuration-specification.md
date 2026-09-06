# T-01342: Init & Service Supervision - Configuration: Specification

## Metadata
- **Task ID:** `T-01342`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Init & Service Supervision Configuration Subsystem Specification (`aiosh-core::service_config`)
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Specification Overview

This document defines the formal contract, data structures, precedence rules, validation invariants (`SC1..SC7`), error envelopes, and audit requirements for the AIOS Init & Service Supervision Configuration Subsystem (`aiosh-core::service_config`).

---

## 2. Reused vs. New Interfaces

### Reused Interfaces:
- `aiosh_core::service::{ServiceSpec, ServiceStatus, ServiceAction, ServiceStartupMode, ServiceState}`: Canonical data types.
- `aiosh_core::service_service::ServiceStore`: Persistent registry and execution engine.
- `aiosh_core::audit::AuditRing`: SQLite WAL audit trail recording configuration queries and updates.
- `aiosh_core::dispatch::recorded_call`: Policy enforcement and audit gate.
- Standard JSON result envelope `{ "ok": bool, ... }`.

### New Interfaces:
- `aiosh_core::service_config::ServiceConfig`: Core configuration struct and resolution loader.
- `aiosh service config [--json] [--config <path>]`: Operator CLI command for inspecting resolved configuration.
- `aios.service.config`: Autonomous Agent MCP tool returning resolved configuration parameters.

---

## 3. Configuration Data Model

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Canonical filesystem path to the persistent service store JSON file.
    pub store_path: PathBuf,
    /// Default execution startup timeout in seconds for services without explicit timeout.
    pub default_timeout_start_secs: u32,
    /// Default execution shutdown timeout in seconds for services without explicit timeout.
    pub default_timeout_stop_secs: u32,
    /// Maximum allowed service store file size on disk (bytes).
    pub max_store_size_bytes: u64,
    /// Maximum service entities permitted within a single store.
    pub max_entity_count: usize,
    /// Whether mutations automatically persist to store_path without explicit flag.
    pub auto_persist: bool,
    /// Throttling backoff delay in seconds between consecutive service restarts.
    pub restart_backoff_secs: u32,
    /// Maximum number of restart attempts permitted within the backoff window.
    pub max_restart_burst: u32,
}
```

### Default Embedded Values:
- `store_path`: `.aios/service_store.json`
- `default_timeout_start_secs`: `30`
- `default_timeout_stop_secs`: `30`
- `max_store_size_bytes`: `10 * 1024 * 1024` (10 MiB)
- `max_entity_count`: `10,000`
- `auto_persist`: `true`
- `restart_backoff_secs`: `5`
- `max_restart_burst`: `5`

---

## 4. Configuration Invariants (`SC1..SC7`)

- **`SC1` (Store Path Validity)**:
  `store_path` must not be empty, must not exceed 1,024 characters, and must not contain ASCII control characters or null bytes (`\0`).
- **`SC2` (Timeout Bounds)**:
  `default_timeout_start_secs` and `default_timeout_stop_secs` must both fall within the valid range $[1 \dots 3,600]$ seconds.
- **`SC3` (Store Size Ceiling Bounds)**:
  `max_store_size_bytes` must fall within $[65,536 \text{ (64 KiB)} \dots 104,857,600 \text{ (100 MiB)}]$.
- **`SC4` (Entity Count Bounds)**:
  `max_entity_count` must fall within $[10 \dots 100,000]$.
- **`SC5` (Restart Throttling Bounds)**:
  `restart_backoff_secs` must be within $[1 \dots 300]$ seconds, and `max_restart_burst` must be within $[1 \dots 50]$ attempts.
- **`SC6` (Resolution Precedence)**:
  Configuration parameters must be resolved strictly in order:
  1. Explicit configuration file path (`--config <path>` or `from_file`).
  2. Environment variables (`AIOS_SERVICE_*`).
  3. Safe embedded defaults (`Default::default()`).
- **`SC7` (Config File Size Ceiling)**:
  Configuration files read from disk must not exceed 65,536 bytes (64 KiB).

---

## 5. Environment Variable Contract

| Variable Name | Type | Description | Default Fallback |
|---|---|---|---|
| `AIOS_SERVICE_STORE_PATH` | String | Path to service store JSON file | `.aios/service_store.json` |
| `AIOS_SERVICE_TIMEOUT_START_SECS` | Integer | Default service startup timeout | `30` |
| `AIOS_SERVICE_TIMEOUT_STOP_SECS` | Integer | Default service stop timeout | `30` |
| `AIOS_SERVICE_MAX_STORE_SIZE_BYTES` | Integer | Maximum store file size in bytes | `10485760` (10 MiB) |
| `AIOS_SERVICE_MAX_ENTITIES` | Integer | Maximum services permitted in store | `10000` |
| `AIOS_SERVICE_AUTO_PERSIST` | Boolean | Automatically write state mutations to disk | `true` |
| `AIOS_SERVICE_RESTART_BACKOFF_SECS` | Integer | Restart backoff delay in seconds | `5` |
| `AIOS_SERVICE_MAX_RESTART_BURST` | Integer | Maximum restart burst attempts | `5` |

---

## 6. Error Modes & Standard Envelope

When configuration validation fails, the subsystem returns a structured failure message:
```json
{
  "ok": false,
  "error": "Configuration validation failed: store_path exceeds 1024 characters or contains control characters (SC1)"
}
```

---

## 7. Acceptance Criteria Verification
- [x] Inputs, outputs, error cases, and persistence effects defined.
- [x] Reused vs new interfaces delineated.
- [x] Invariants `SC1..SC7` formally codified with exact numeric bounds.
- [x] Environment variable mappings and resolution precedence specified.
