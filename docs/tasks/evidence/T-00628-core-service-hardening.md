# T-00628 — Repository Health / core service: Hardening

## 1. Hardening Scope
This task verifies memory bounds, directory exclusion heuristics, execution duration metrics, and fail-closed error containment in `aiosh-core::repo_health_service`.

## 2. Hardening Safeguards & Invariants
- **Bounded Diagnostics**:
  - Uncommitted git changes and oversized file listings in `details` are capped at $\le 50$ entries via `.take(50)` to prevent unbounded memory allocation.
- **Directory Traversal Exclusions**:
  - Scanning routines explicitly ignore build and dependency caches (`.git`, `target`, `node_modules`, `.venv`), avoiding false positives and excessive traversal latency.
- **Execution Duration Tracking**:
  - Every individual check tracks its execution time with `Instant::now()` and records `duration_ms`.
- **Fail-Closed Result Safety**:
  - Subprocess execution or I/O errors never panic and are captured into structured `HealthStatus::Warn` or `HealthStatus::Fail` diagnostic objects.

## 3. Test Verification Output
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.44s
     Running unittests src\lib.rs (code\aiosh-rust\target\debug\deps\aiosh_core-04423105fe4b6d57.exe)

running 6 tests
test repo_health_service::tests::test_check_security_governance_missing ... ok
test repo_health_service::tests::test_check_file_bounds_oversized ... ok
test repo_health_service::tests::test_check_file_bounds_happy ... ok
test repo_health_service::tests::test_check_repo_health_orchestrator ... ok
test repo_health_service::tests::test_check_security_governance_valid ... ok
test repo_health_service::tests::test_check_security_governance_todo_markers ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.02s
```
