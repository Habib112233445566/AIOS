# T-00695 — Repository Health / documentation: Unit Test

## 1. Test Scope
This task adds unit tests for `format_repo_health_summary` in `code/aiosh-rust/aiosh-core/src/repo_health_service.rs` covering:
- Standard populated report with pass and warn statuses (`test_format_repo_health_summary`).
- Empty checks boundary condition and fail/skip multi-status reports (`test_format_repo_health_summary_empty_and_fail`).

## 2. Test Verification
```text
running 8 tests
test repo_health_service::tests::test_check_file_bounds_happy ... ok
test repo_health_service::tests::test_check_file_bounds_oversized ... ok
test repo_health_service::tests::test_check_repo_health_orchestrator ... ok
test repo_health_service::tests::test_check_security_governance_missing ... ok
test repo_health_service::tests::test_check_security_governance_todo_markers ... ok
test repo_health_service::tests::test_check_security_governance_valid ... ok
test repo_health_service::tests::test_format_repo_health_summary ... ok
test repo_health_service::tests::test_format_repo_health_summary_empty_and_fail ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 164 filtered out; finished in 0.02s
```
