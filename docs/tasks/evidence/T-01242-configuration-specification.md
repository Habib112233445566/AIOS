# T-01242: Package Management - Configuration: Specification

## Metadata
- **Task ID:** `T-01242`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Package Management Configuration Subsystem Specification
- **Status:** Complete

## 1. Specification Overview
This document specifies the exact contract, data structures, precedence rules, validation invariants, error modes, and audit requirements for the AIOS Package Management Configuration Subsystem (`aiosh-core::package_config`).

## 2. Reused vs. New Interfaces

### Reused Interfaces:
- `aiosh_core::package::PackageFormat`: Enum defining supported formats (`deb`, `apk`, `flatpak`, `tarball`).
- `aiosh_core::audit`: `AuditRing` and audit row emission for tracking configuration access.
- `aiosh_core::dispatch::recorded_call`: MCP authorization gating and outcome tracking.
- Standard JSON result envelope `{ "ok": bool, "data": ..., "error": ... }`.

### New Interfaces:
- `aiosh_core::package_config::PackageConfig`: Main configuration struct and loader.
- `aiosh package config [--json] [--config <path>]`: Operator CLI command for inspecting configuration.
- `aios.package.config`: Autonomous Agent MCP tool returning resolved configuration.

---

## 3. Configuration Data Model

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Canonical filesystem path to the persistent package store JSON file.
    pub store_path: PathBuf,
    /// Default packaging format used when format filter/target is omitted.
    pub default_format: PackageFormat,
    /// Maximum allowed package store file size on disk (bytes).
    pub max_store_size_bytes: u64,
    /// Maximum package entities permitted within a single store.
    pub max_entity_count: usize,
    /// Whether mutations automatically persist to store_path without explicit flag.
    pub auto_persist: bool,
    /// List of trusted HTTPS repository upstream URLs.
    pub allowed_repositories: Vec<String>,
}
```

### Default Embedded Values:
- `store_path`: `.aios/packages.json`
- `default_format`: `PackageFormat::Deb`
- `max_store_size_bytes`: `10 * 1024 * 1024` (10 MiB)
- `max_entity_count`: `10,000`
- `auto_persist`: `false`
- `allowed_repositories`:
  - `"https://deb.debian.org/debian"`
  - `"https://dl-cdn.alpinelinux.org/alpine/v3.19/main"`

---

## 4. Configuration Invariants (`PC1..PC6`)

- **`PC1` (Store Path Validity)**:
  `store_path` must not be empty, must be $\le 1,024$ bytes, and must not contain ASCII control characters or null bytes (`\0`).
- **`PC2` (Store Size Ceiling Bounds)**:
  `max_store_size_bytes` must be within the valid range $[65,536 \text{ (64 KiB)} \dots 104,857,600 \text{ (100 MiB)}]$.
- **`PC3` (Entity Count Bounds)**:
  `max_entity_count` must be within the valid range $[10 \dots 100,000]$.
- **`PC4` (Repository Transport Security)**:
  All strings in `allowed_repositories` must begin with `https://` (or `file://` for air-gapped / loopback chroot mirrors). Plaintext `http://` is strictly prohibited.
- **`PC5` (Resolution Precedence)**:
  Resolution order is strictly:
  1. Explicit configuration file path (`--config <path>` or `from_file`).
  2. Environment variables (`AIOS_PACKAGE_*`).
  3. Embedded defaults (`Default::default()`).
- **`PC6` (Config File Size Ceiling)**:
  Configuration files read from disk must not exceed 65,536 bytes (64 KiB).

---

## 5. Environment Variable Contract

| Variable Name | Type | Description | Fallback |
|---|---|---|---|
| `AIOS_PACKAGE_STORE_PATH` | String | Path to package store JSON | `.aios/packages.json` |
| `AIOS_PACKAGE_DEFAULT_FORMAT` | String | Default format (`deb`, `apk`, etc.) | `deb` |
| `AIOS_PACKAGE_MAX_STORE_SIZE_BYTES`| Integer | Max store file size in bytes | `10485760` (10 MiB) |
| `AIOS_PACKAGE_MAX_ENTITIES` | Integer | Max packages allowed in store | `10000` |
| `AIOS_PACKAGE_AUTO_PERSIST` | Boolean | Automatic store persistence flag | `false` |
| `AIOS_PACKAGE_ALLOWED_REPOS` | String | Comma-separated HTTPS repository URLs | Built-in defaults |

---

## 6. Error Modes & Standard Envelope

When configuration validation fails, the subsystem returns a structured error:
```json
{
  "code": 2,
  "data": null,
  "error": {
    "code": "INVALID_CONFIGURATION",
    "message": "PC4 violation: repository 'http://insecure.example.com' must use HTTPS"
  }
}
```

---

## 7. Audit Effects (ADR-0035)
1. Configuration resolution via CLI or MCP emits a classified audit record into the SQLite WAL ring (`AuditRing`).
2. Event record includes actor ID, target config path, and SHA-256 fingerprint of the active configuration.
