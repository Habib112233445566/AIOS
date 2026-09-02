# T-00622 — Repository Health / core service: Specification

## 1. Specification Overview
The `aiosh-core::repo_health_service` provides high-level health checking functions that inspect a local repository checkout and construct validated `RepoHealthReport` objects.

## 2. API Contract & Function Signatures

### 2.1 `check_git_working_tree`
```rust
pub fn check_git_working_tree(repo_root: &Path) -> RepoHealthCheck;
```
- **Inputs**: `repo_root: &Path` — path to repository working directory.
- **Behavior**:
  - Invokes `git status --porcelain=v2` in `repo_root` with a 5-second process timeout.
  - If exit code == 0 and output contains 0 entries (`1`, `2`, `u`, `?`), returns `HealthStatus::Pass`.
  - If untracked (`?`) or uncommitted changes (`1`, `2`, `u`) exist, returns `HealthStatus::Warn` with entry counts in `message`.
  - If `git` is not found or fails to execute, returns `HealthStatus::Warn` or `HealthStatus::Fail` with explicit error message.
- **Output**: `RepoHealthCheck` (`category = HealthCategory::GitHygiene`).

### 2.2 `check_file_bounds`
```rust
pub fn check_file_bounds(repo_root: &Path, max_bytes: u64) -> RepoHealthCheck;
```
- **Inputs**: `repo_root: &Path`, `max_bytes: u64` (default 16 MiB / 16,777,216 bytes).
- **Behavior**:
  - Recursively traverses `repo_root` skipping `.git/` and `target/`.
  - Asserts every file size $\le \text{max\_bytes}$.
  - If any file exceeds `max_bytes`, returns `HealthStatus::Fail` listing oversized files in `details`.
  - If all files within bounds, returns `HealthStatus::Pass`.
- **Output**: `RepoHealthCheck` (`category = HealthCategory::FileIntegrity`).

### 2.3 `check_security_governance`
```rust
pub fn check_security_governance(repo_root: &Path) -> RepoHealthCheck;
```
- **Inputs**: `repo_root: &Path`.
- **Behavior**:
  - Checks existence of `SECURITY.md` at `repo_root`.
  - Asserts file size $> 100$ bytes and contains no unresolved `TODO` markers.
  - Returns `HealthStatus::Pass` on valid security policy, else `HealthStatus::Fail`.
- **Output**: `RepoHealthCheck` (`category = HealthCategory::SecurityGovernance`).

### 2.4 `check_repo_health`
```rust
pub fn check_repo_health(repo_root: &Path) -> Result<RepoHealthReport, String>;
```
- **Inputs**: `repo_root: &Path`.
- **Behavior**:
  - Executes all diagnostic checks (`check_git_working_tree`, `check_file_bounds`, `check_security_governance`).
  - Formats current UTC timestamp (`chrono::Utc::now().to_rfc3339()`).
  - Constructs and validates `RepoHealthReport::new(...)`.
- **Errors**: Returns `Err(String)` if input path is invalid or report validation fails.

## 3. Error Handling & Invariants
- All functions are non-panicking and fail-closed.
- Any I/O or subprocess failure is encapsulated into a structured `RepoHealthCheck` result.
