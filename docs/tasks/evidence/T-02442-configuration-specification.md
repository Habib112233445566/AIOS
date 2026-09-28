# T-02442: Sandbox Enforcement Configuration Specification

## 1. Specification Overview
This specification defines the configuration contract, schema, validation constraints, file persistence, and environment variable overrides for the Sandbox Enforcement subsystem.

---

## 2. Configuration Schema & Types

```rust
pub struct SandboxConfig {
    pub version: String,
    pub default_profile_name: String,
    pub max_output_capture_bytes: usize,
    pub execution_timeout_seconds: u64,
    pub max_registered_profiles: usize,
    pub enforce_pep_grants: bool,
    pub audit_enabled: bool,
    pub custom_profiles_dir: Option<PathBuf>,
}
```

---

## 3. Boundary Constants & Limits
- `MAX_CONFIG_FILE_BYTES = 64 * 1024` (64 KiB)
- `MIN_OUTPUT_CAPTURE_BYTES = 1024` (1 KiB)
- `MAX_OUTPUT_CAPTURE_BYTES = 64 * 1024 * 1024` (64 MiB, default: 10 MiB)
- `MIN_TIMEOUT_SECONDS = 1`
- `MAX_TIMEOUT_SECONDS = 86400` (24h, default: 300s)
- `MIN_REGISTERED_PROFILES = 1`
- `MAX_REGISTERED_PROFILES_CAP = 1024` (default: 256)

---

## 4. Error Codes
- `SANDBOXCONF_ERR_IO`: Filesystem reading or writing error.
- `SANDBOXCONF_ERR_PARSE`: JSON syntax or deserialization failure.
- `SANDBOXCONF_ERR_VALIDATION`: Version mismatch or invalid profile name.
- `SANDBOXCONF_ERR_BOUNDS`: Numerical parameter outside permissible bounds.

---

## 5. Environment Variable Overrides
- `AIOS_SANDBOX_DEFAULT_PROFILE`: Overrides default profile name.
- `AIOS_SANDBOX_ENFORCE_PEP`: "true" / "1" enables mandatory PEP capability gating.
- `AIOS_SANDBOX_MAX_OUTPUT_BYTES`: Integer override for process output buffer size.
- `AIOS_SANDBOX_TIMEOUT_SECS`: Integer override for execution wall-clock timeout.
- `AIOS_SANDBOX_PROFILES_DIR`: Directory path for loading custom profile definitions.
