# T-00624 — Repository Health / core service: Implementation

## 1. Implementation Scope
This task implements the core repository health checking service in `aiosh-core::repo_health_service`.

## 2. Implemented Functions
1. **`check_git_working_tree(repo_root: &Path) -> RepoHealthCheck`**:
   - Executes `git status --porcelain=v2` in the target repository directory.
   - Parses change entries (`1`, `2`, `u`, `?`), ignoring header comments (`#`).
   - Returns `HealthStatus::Pass` if 0 changes, or `HealthStatus::Warn` detailing uncommitted or untracked modifications.
2. **`check_file_bounds(repo_root: &Path, max_bytes: u64) -> RepoHealthCheck`**:
   - Recursively walks repository files skipping `.git`, `target`, `node_modules`, and `.venv`.
   - Flags any file exceeding `max_bytes` (default 16 MiB).
   - Returns `HealthStatus::Pass` if within limits, or `HealthStatus::Fail` with a list of oversized files.
3. **`check_security_governance(repo_root: &Path) -> RepoHealthCheck`**:
   - Validates existence, minimum length ($\ge 100$ characters), and absence of unresolved `TODO` markers in `SECURITY.md`.
4. **`check_repo_health(repo_root: &Path) -> Result<RepoHealthReport, String>`**:
   - Orchestrates all checks, timestamps execution, and produces a validated aggregate `RepoHealthReport`.

## 3. Test Verification Output
```text
running 6 tests
test repo_health_service::tests::test_check_security_governance_missing ... ok
test repo_health_service::tests::test_check_file_bounds_oversized ... ok
test repo_health_service::tests::test_check_file_bounds_happy ... ok
test repo_health_service::tests::test_check_security_governance_todo_markers ... ok
test repo_health_service::tests::test_check_repo_health_orchestrator ... ok
test repo_health_service::tests::test_check_security_governance_valid ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.04s
```
