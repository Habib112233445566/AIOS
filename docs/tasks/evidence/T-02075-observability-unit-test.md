# Evidence: T-02075 - observability: Unit Test

## Task Overview
- **Task ID**: `T-02075`
- **Sub-Epic**: Sub-Epic 8: Observability (`T-02071`..`T-02080`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Add focused automated unit tests for Capability Model Observability (`CAPOBS1..CAPOBS6`).

## Unit Test Suite (`code/aiosh-rust/aiosh-core/tests/test_capability_observability.rs`)
1. `test_observability_empty_registry`: Validates report defaults on empty registry (0 counts, 0 utilization, `is_healthy: true`).
2. `test_observability_populated_registry`: Validates report with root and attenuated capabilities across filesystem and network scopes, checking quota sums, depth tracking, and scope/rights distributions.
3. `test_observability_revocation_and_expiration`: Validates accurate separation of active, expired, and revoked capabilities.
4. `test_observability_health_evaluation`: Validates `is_healthy` flag under normal conditions and asserts failure when derivation depth exceeds policy limits.
5. `test_observability_json_serde_and_sanitization`: Validates JSON serialization round-trip and tests `sanitize_telemetry_text()` control character stripping and length capping.

## Test Results
```text
running 5 tests
test test_observability_empty_registry ... ok
test test_observability_json_serde_and_sanitization ... ok
test test_observability_populated_registry ... ok
test test_observability_health_evaluation ... ok
test test_observability_revocation_and_expiration ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```
