# T-02275 Unit Test: Grant Lifecycle Observability

**Task:** Add focused automated tests for the observability of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Observability  

---

## 1. Unit Tests Created

Created `code/aiosh-rust/aiosh-core/tests/test_pep_grant_observability.rs` with 6 focused test functions covering positive, negative, and boundary vectors:

| Test Name | Vector Tested | Behavior Asserted |
|---|---|---|
| `test_grant_observability_empty_service` | Empty registry state | Zero counts, zero utilization, is_healthy=true |
| `test_grant_observability_populated_service` | Multi-state aggregation | Active, suspended, revoked counts match, root vs derived breakdown, unique subjects |
| `test_grant_observability_invariant_validation` | Report consistency | Invariant failure when total count does not match state sum (`PEPOBS_GRANT_ERR_VALIDATION`) |
| `test_grant_observability_health_threshold_flip` | SRE Golden Signals | Health indicator flips to false when capacity utilization >= 90% |
| `test_grant_observability_store_integration` | `PepGrantStore` integration | `PepGrantStore::generate_observability_report` compiles and validates cleanly |
| `test_grant_telemetry_sanitization` | Telemetry security | Strips non-printable/control characters, enforces length limits (max 256 chars) |

---

## 2. Acceptance Verification
- ✅ Standalone test file created under `tests/`.
- ✅ All 6 tests pass asserting observable telemetry outputs and invariant checks.
- ✅ Negative error cases (`PEPOBS_GRANT_ERR_VALIDATION`) explicitly verified.
