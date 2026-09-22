# Task Evidence: T-02175 - PEP Decision Engine: Observability: Unit Test

## Task Metadata
- **Task ID**: `T-02175`
- **Sub-Epic**: Sub-Epic 8: Observability Subsystem
- **Component**: `aiosh-core::pep_observability`
- **Date**: 2026-09-22
- **Status**: Completed

## 1. Test Coverage Strategy
Unit tests for the PEP Decision Engine Observability Subsystem (`code/aiosh-rust/aiosh-core/tests/test_pep_observability.rs`) verify:
1. **Empty Service Baseline**:
   - Total rules = 0, utilization = 0%, healthy = true.
   - Initial effect and obligation buckets properly zero-initialized.
   - Serialization and structural invariants pass.
2. **Populated Service Aggregation**:
   - Addition of rules with diverse effects (`Permit`, `Deny`) and obligations (`AuditLog`, `RateLimit`, `RedactFields`, `Custom`).
   - Verifies exact counts in `rules_by_effect`, `obligations_by_type`, `unique_subjects_count`, `unique_resources_count`, `unique_actions_count`.
3. **Capacity Threshold & Health Degradation**:
   - `PEP_HEALTH_UTILIZATION_THRESHOLD` (90%).
   - At 89% utilization -> `is_healthy == true`.
   - At 90% and 95% utilization -> `is_healthy == false`.
4. **Telemetry Sanitization**:
   - Stripping of ASCII control characters (`\x00`, `\x07`, `\r`, `\n`, `\t`).
   - Length cap enforcement at 256 characters.
   - Whitespace trimming.
5. **Validation Invariant Enforcement**:
   - `total_rules > capacity_limit` -> rejects with `PEPOBS_ERR_VALIDATION`.
   - `capacity_utilization_percent > 100` -> rejects with `PEPOBS_ERR_VALIDATION`.
   - `sum(rules_by_effect) != total_rules` -> rejects with `PEPOBS_ERR_VALIDATION`.
   - `rules_with_obligations > total_rules` -> rejects with `PEPOBS_ERR_VALIDATION`.
   - Blank `generated_at` -> rejects with `PEPOBS_ERR_VALIDATION`.
6. **JSON Serialization / Deserialization Roundtrip**:
   - Tests `to_json()` pretty printing and `from_json()` deserialization idempotency.
7. **Timestamp Fallback**:
   - Empty input string falls back to current valid RFC3339 UTC timestamp.

## 2. Test Execution
All 7 unit tests pass with zero failures:
- `test_pep_observability_empty_service` ... ok
- `test_pep_observability_populated_service` ... ok
- `test_pep_observability_health_utilization_threshold` ... ok
- `test_pep_observability_sanitization` ... ok
- `test_pep_observability_validation_invariants` ... ok
- `test_pep_observability_json_roundtrip` ... ok
- `test_pep_observability_timestamp_fallback` ... ok
