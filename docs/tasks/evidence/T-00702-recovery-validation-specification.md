# T-00702 — Repository Health / recovery & validation: Specification

## 1. Specification Overview
This specification defines the recovery mechanisms, default reconstruction routines, structural report validators, and automated reconciliation workflows for Repository Health in AIOS.

## 2. Recovery & Validation Function Contracts

### A. Default Configuration Recovery
```rust
pub fn recover_default_repo_health_config() -> RepoHealthConfig
```
- **Inputs**: None.
- **Outputs**: Returns a canonical in-memory `RepoHealthConfig` with safe default parameters:
  - `version`: `"1.0.0"`
  - `max_file_bytes`: `16,777,216` (16 MiB)
  - `ignored_dirs`: `[".git", "target", "node_modules", ".venv"]`
  - `require_clean_git`: `false`
  - `security_policy_path`: `"SECURITY.md"`
  - `min_security_policy_bytes`: `100`

### B. Report Reconstruction
```rust
pub fn reconstruct_repo_health_report(repo_root: &Path, config: &RepoHealthConfig) -> Result<RepoHealthReport, String>
```
- **Inputs**: `repo_root` path reference, `config` reference.
- **Behavior**:
  - Validates `repo_root` exists on disk.
  - Executes git working tree check (`check_git_working_tree`).
  - Executes file bounds scan honoring `config.max_file_bytes`.
  - Executes security governance audit honoring `config.security_policy_path` and `config.min_security_policy_bytes`.
  - Synthesizes and validates `RepoHealthReport`.
- **Errors**: Returns `Err(String)` if root path is invalid or report validation fails.

### C. Report Structural Validation
```rust
pub fn validate_repo_health_report(report: &RepoHealthReport) -> Result<(), String>
```
- **Inputs**: `report` reference.
- **Behavior**: Calls `report.validate()`, verifying path bounds, timestamp presence, check count arithmetic (`total == pass + warn + fail + skip`), and status derivation.
- **Errors**: Returns `Err(String)` if any invariant fails.

### D. Automated Reconciliation
```rust
pub fn reconcile_repo_health(repo_root: &Path) -> Result<RepoHealthReport, String>
```
- **Inputs**: `repo_root` path reference.
- **Behavior**:
  - Attempts to load configuration via `RepoHealthConfig::from_env()`.
  - Falls back to `recover_default_repo_health_config()` if configuration cannot be resolved.
  - Invokes `reconstruct_repo_health_report(repo_root, &config)`.
  - Validates the resulting report via `validate_repo_health_report(&report)`.
  - Returns the reconciled `RepoHealthReport`.

## 3. Error Handling & Invariant Guarantees
- Failures in individual checks (e.g. non-zero git exit code) populate warning/error check records rather than panicking.
- Read operations are strictly bounded to prevent infinite recursion or out-of-bounds filesystem escapes.
- Recovery routines never perform mutating file deletions or uncommitted git resets.
