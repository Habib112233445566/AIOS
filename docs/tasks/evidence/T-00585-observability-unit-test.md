# T-00585 — Evidence & Audit Trail / observability: Unit Test

## 1. Unit Test Scope
This task implements and executes comprehensive unit tests for `EvidenceTelemetry` and `collect_evidence_telemetry` in `code/aiosh-rust/aiosh-core`.

## 2. Test Cases & Coverage
1. **Happy Path Report**:
   - 10 total records, 10 valid records, 0 missing, 0 mismatches $\to$ `is_healthy: true`.
2. **Degraded Report**:
   - 10 total records, 7 valid records, 2 missing, 1 mismatch $\to$ `is_healthy: false`, counts match.
3. **Empty Boundary Report**:
   - 0 total records, 0 valid $\to$ `is_healthy: true`.
4. **All-Missing Boundary**:
   - 3 total records, 3 missing $\to$ `is_healthy: false`.
5. **JSON Serialization & Deserialization**:
   - Roundtrip validation asserting `EvidenceTelemetry` preserves schema across serde cycles.

## 3. Test Verification Output
```text
running 1 test
test evidence_service::tests::test_collect_evidence_telemetry ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 152 filtered out; finished in 0.00s
```
