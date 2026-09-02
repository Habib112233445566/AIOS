# T-00703 — Repository Health / recovery & validation: Scaffold

## 1. Scaffold Scope
This task defines typed function signatures and interface definitions for recovery and validation routines in `code/aiosh-rust/aiosh-core/src/repo_health_service.rs`.

## 2. Scaffold Deliverables
- **`code/aiosh-rust/aiosh-core/src/repo_health_service.rs`**:
  - `recover_default_repo_health_config() -> RepoHealthConfig`
  - `reconstruct_repo_health_report(repo_root: &Path, config: &RepoHealthConfig) -> Result<RepoHealthReport, String>`
  - `validate_repo_health_report(report: &RepoHealthReport) -> Result<(), String>`
  - `reconcile_repo_health(repo_root: &Path) -> Result<RepoHealthReport, String>`

## 3. Build Verification
Compilation verified cleanly with zero compiler errors across `aiosh-core`.
