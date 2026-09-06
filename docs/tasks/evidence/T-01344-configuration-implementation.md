# T-01344: Init & Service Supervision - Configuration: Implementation

## Metadata
- **Task ID:** `T-01344`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Init & Service Supervision Configuration Implementation (`aiosh-core::service_config`)
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Implementation Summary

This task implements the minimal working behavior and validation invariants `SC1..SC7` for the Init & Service Supervision Configuration Subsystem in `code/aiosh-rust/aiosh-core/src/service_config.rs`.

### Implemented Capabilities:
1. **`ServiceConfig::validate(&self) -> Result<(), String>`**:
   - `SC1`: Store path non-empty, max 1024 bytes, no control chars or null bytes.
   - `SC2`: Timeout bounds verification for both startup and stop timeouts within $[1 \dots 3600]$ seconds.
   - `SC3`: Store size ceiling bounds within $[65,536 \text{ (64 KiB)} \dots 104,857,600 \text{ (100 MiB)}]$.
   - `SC4`: Entity count ceiling within $[10 \dots 100,000]$.
   - `SC5`: Restart backoff delay $[1 \dots 300]$ seconds and burst attempts $[1 \dots 50]$.

2. **`ServiceConfig::from_file(path: &Path) -> Result<Self, String>`**:
   - Implements `SC7` bounded configuration file read (capped at 64 KiB).
   - Deserializes JSON and invokes `validate()`.

3. **`ServiceConfig::from_env() -> Result<Self, String>`**:
   - Inspects `AIOS_SERVICE_STORE_PATH`, `AIOS_SERVICE_TIMEOUT_START_SECS`, `AIOS_SERVICE_TIMEOUT_STOP_SECS`, `AIOS_SERVICE_MAX_STORE_SIZE_BYTES`, `AIOS_SERVICE_MAX_ENTITIES`, `AIOS_SERVICE_AUTO_PERSIST`, `AIOS_SERVICE_RESTART_BACKOFF_SECS`, and `AIOS_SERVICE_MAX_RESTART_BURST`.
   - Validates merged parameters.

4. **`ServiceConfig::resolve(config_path_opt: Option<&Path>) -> Result<Self, String>`**:
   - Enforces precedence rule `SC6` (CLI `--config` > `AIOS_SERVICE_CONFIG` > environment variables > default safe embedded values).

---

## 2. Test Verification

Tests executed in `aiosh-core`:
- `test_service_config_default_and_validation`: PASS
- `test_service_config_sc1_store_path_invariants`: PASS
- `test_service_config_sc2_timeout_invariants`: PASS
- `test_service_config_sc3_sc4_sc5_boundary_invariants`: PASS
- `test_service_config_file_roundtrip_and_sc7`: PASS

---

## 3. Acceptance Verification
- [x] Targeted tests pass.
- [x] No regressions in existing modules.
- [x] Invariants `SC1..SC7` verified and enforced.
