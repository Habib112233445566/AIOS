# T-00625 — Repository Health / core service: Unit Test

## 1. Unit Test Scope
This task verifies the functionality of `aiosh-core::repo_health_service` using isolated automated unit tests.

## 2. Test Execution & Coverage
1. **`test_check_file_bounds_happy`**:
   - Asserts files within size limit evaluate to `HealthStatus::Pass`.
2. **`test_check_file_bounds_oversized`**:
   - Asserts files exceeding size limit evaluate to `HealthStatus::Fail` and return formatted file list in `details`.
3. **`test_check_security_governance_missing`**:
   - Asserts missing `SECURITY.md` evaluates to `HealthStatus::Fail`.
4. **`test_check_security_governance_valid`**:
   - Asserts valid `SECURITY.md` evaluates to `HealthStatus::Pass`.
5. **`test_check_security_governance_todo_markers`**:
   - Asserts `SECURITY.md` containing unresolved `TODO` markers evaluates to `HealthStatus::Fail`.
6. **`test_check_repo_health_orchestrator`**:
   - Asserts multi-check aggregation produces a valid 3-check `RepoHealthReport`.

## 3. Test Verification Output
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.69s
     Running unittests src\lib.rs (code\aiosh-rust\target\debug\deps\aiosh_core-04423105fe4b6d57.exe)

running 6 tests
test repo_health_service::tests::test_check_security_governance_missing ... ok
test repo_health_service::tests::test_check_file_bounds_happy ... ok
test repo_health_service::tests::test_check_repo_health_orchestrator ... ok
test repo_health_service::tests::test_check_file_bounds_oversized ... ok
test repo_health_service::tests::test_check_security_governance_valid ... ok
test repo_health_service::tests::test_check_security_governance_todo_markers ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.02s
```
