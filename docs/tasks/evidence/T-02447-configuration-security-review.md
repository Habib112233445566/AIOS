# T-02447: Sandbox Enforcement Configuration Security Review

## 1. Review Scope
Security assessment of `SandboxConfig` in `code/aiosh-rust/aiosh-core/src/sandbox_config.rs`.

## 2. Threat Modeling & Abuse Scenarios

### Scenario A: Unbounded File Read / Disk Bomb (CWE-400)
- **Vector**: Operator or attacker points `load_from_path` to a device node or multi-gigabyte file (e.g., `/dev/urandom` or large logs).
- **Defense**: File metadata is checked prior to reading. Files exceeding `MAX_CONFIG_FILE_BYTES = 64 KiB` fail immediately with `SANDBOXCONF_ERR_BOUNDS`.

### Scenario B: Directory Traversal via Custom Profiles Directory (CWE-22)
- **Vector**: Supplying `custom_profiles_dir: "../../etc/shadow"` to escape sandbox configuration boundaries.
- **Defense**: Validation explicitly scans `custom_profiles_dir` and fails closed if any component contains `..` (`SANDBOXCONF_ERR_VALIDATION`).

### Scenario C: Extreme Numerical Bounds (CWE-1284)
- **Vector**: Setting `max_output_capture_bytes: 0` (causing unexpected panic) or `usize::MAX` (causing OOM), or `execution_timeout_seconds: 0`.
- **Defense**: Clamped numerical limits:
  - `max_output_capture_bytes`: `1 KiB .. 64 MiB`
  - `execution_timeout_seconds`: `1s .. 86,400s`
  - `max_registered_profiles`: `1 .. 1024`

### Scenario D: State Inconsistency via Hostile Environment Variables
- **Vector**: Injecting non-numeric or out-of-bounds strings in `AIOS_SANDBOX_TIMEOUT_SECS` or `AIOS_SANDBOX_MAX_OUTPUT_BYTES`.
- **Defense**: `load_with_env_overrides` validates parsed integers against boundary constants, falling back safely to factory defaults on invalid or out-of-range strings.

## 3. Finding & Certification
- **Vulnerabilities**: 0 Critical, 0 High, 0 Medium, 0 Low.
- **Certification**: **APPROVED / PASS**.
