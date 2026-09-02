# T-00704 — Repository Health / recovery & validation: Implementation

## 1. Implementation Scope
This task implements recovery and validation functions (`recover_default_repo_health_config`, `reconstruct_repo_health_report`, `validate_repo_health_report`, `reconcile_repo_health`) in `code/aiosh-rust/aiosh-core/src/repo_health_service.rs`.

## 2. Implementation Deliverables
- **`code/aiosh-rust/aiosh-core/src/repo_health_service.rs`**:
  - `recover_default_repo_health_config`: In-memory recovery of canonical defaults.
  - `reconstruct_repo_health_report`: Rebuilds full health report from disk state.
  - `validate_repo_health_report`: Verifies report structural and arithmetic invariants.
  - `reconcile_repo_health`: Coordinates configuration resolution, reconstruction, and validation.
  - Unit tests `test_recover_default_repo_health_config` and `test_reconstruct_and_reconcile_repo_health`.

## 3. Test Verification
```text
running 11 tests
test repo_health_service::tests::test_check_file_bounds_happy ... ok
test repo_health_service::tests::test_check_file_bounds_oversized ... ok
test repo_health_service::tests::test_check_repo_health_orchestrator ... ok
test repo_health_service::tests::test_check_security_governance_missing ... ok
test repo_health_service::tests::test_check_security_governance_todo_markers ... ok
test repo_health_service::tests::test_check_security_governance_valid ... ok
test repo_health_service::tests::test_format_repo_health_summary ... ok
test repo_health_service::tests::test_format_repo_health_summary_empty_and_fail ... ok
test repo_health_service::tests::test_format_repo_health_summary_truncation ... ok
test repo_health_service::tests::test_recover_default_repo_health_config ... ok
test repo_health_service::tests::test_reconstruct_and_reconcile_repo_health ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 164 filtered out; finished in 0.05s
```
