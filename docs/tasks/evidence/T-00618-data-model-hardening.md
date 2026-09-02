# T-00618 — Repository Health / data model: Hardening

## 1. Hardening Scope
This task verifies defensive invariants, allocation limits, and fail-closed error handling in `aiosh-core::repo_health`.

## 2. Hardening Invariants & Defenses
- **String Bounds**:
  - `check_id`: 1..64 chars (alphanumeric, hyphens, underscores).
  - `name`: 1..128 chars.
  - `message`: $\le 1024$ chars.
  - `details`: List length $\le 100$, each detail entry $\le 512$ chars.
  - `repo_path`: 1..1024 chars.
- **Mathematical Invariant**:
  - `total_checks == passed_checks + warn_checks + failed_checks + skipped_checks == checks.len()`.
- **Deterministic Status Resolution**:
  - `overall_status` strictly derived from sub-check statuses with fail-closed precedence.
- **Fail-Closed Result Type**:
  - All validations return structured `Result<(), String>` with detailed contextual errors.

## 3. Test Verification Output
```text
running 5 tests
test repo_health::tests::test_repo_health_check_validation_errors ... ok
test repo_health::tests::test_repo_health_check_validation_happy ... ok
test repo_health::tests::test_repo_health_report_happy_and_status_derivation ... ok
test repo_health::tests::test_repo_health_report_json_roundtrip ... ok
test repo_health::tests::test_repo_health_report_validation_errors ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; finished in 0.00s
```
