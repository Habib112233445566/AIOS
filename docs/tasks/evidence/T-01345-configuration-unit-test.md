# T-01345: Init & Service Supervision - Configuration: Unit Test

## Metadata
- **Task ID:** `T-01345`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** Init & Service Supervision Configuration Unit Test Suite (`test_service_config.rs`)
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Test Suite Overview

Added dedicated automated unit test suite `code/aiosh-rust/aiosh-core/tests/test_service_config.rs` covering all positive, negative, boundary, and file size invariants (`SC1..SC7`) for the Init & Service Supervision Configuration Subsystem.

---

## 2. Test Coverage Matrix

| Test Function | Invariant | Description & Assertion |
|---|---|---|
| `test_service_config_defaults_and_validation` | Baseline | Verifies default values and that `validate()` returns `Ok(())`. |
| `test_service_config_sc1_store_path_invariants` | `SC1` | Rejects empty path, paths > 1024 bytes, null bytes `\0`, and control characters `\n`. Accepts exact 1024 bytes. |
| `test_service_config_sc2_timeout_invariants` | `SC2` | Validates startup and stop timeouts within $[1 \dots 3600]$s. Rejects 0 and 3601s. |
| `test_service_config_sc3_sc4_sc5_boundary_invariants` | `SC3`, `SC4`, `SC5` | Rejects store size $< 64\text{ KiB}$ and $> 100\text{ MiB}$. Rejects entity counts $< 10$ and $> 100,000$. Rejects restart backoff $< 1$s and $> 300$s. Rejects restart burst $< 1$ and $> 50$. |
| `test_service_config_sc6_env_resolution` | `SC6` | Overrides all fields through environment variables `AIOS_SERVICE_*` and asserts parsed values. |
| `test_service_config_sc7_file_roundtrip_and_size_cap` | `SC7`, `SC6` | JSON serialization/deserialization roundtrip, `resolve(Some(path))`, and rejection of configuration files $> 64\text{ KiB}$. |

---

## 3. Verification & Acceptance Criteria
- [x] Dedicated test file `code/aiosh-rust/aiosh-core/tests/test_service_config.rs` created and runs standalone.
- [x] Negative, boundary, and error cases asserted across all invariants `SC1..SC7`.
- [x] Validated observable behavior (return values, errors, file parsing).
