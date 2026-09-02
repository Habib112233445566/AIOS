# T-00584 — Evidence & Audit Trail / observability: Implementation

## 1. Implementation Scope
This task implements the telemetry collection helper `collect_evidence_telemetry` and unit test coverage for Evidence & Audit Trail observability in `code/aiosh-rust/aiosh-core`.

## 2. Implementation Deliverables
- **`code/aiosh-rust/aiosh-core/src/evidence.rs`**:
  - `EvidenceTelemetry` struct with `total_records`, `valid_records`, `missing_files_count`, `hash_mismatches_count`, and `is_healthy`.
- **`code/aiosh-rust/aiosh-core/src/evidence_service.rs`**:
  - `collect_evidence_telemetry(&EvidenceVerificationReport) -> EvidenceTelemetry`.
  - Unit test `test_collect_evidence_telemetry` verifying both healthy and degraded verification reports.

## 3. Test Verification
```text
running 1 test
test evidence_service::tests::test_collect_evidence_telemetry ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 152 filtered out; finished in 0.01s
```
