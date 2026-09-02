# T-00623 — Repository Health / core service: Scaffold

## 1. Scaffold Scope
This task creates the module skeleton and typed interfaces for `aiosh-core::repo_health_service` and exports it in `lib.rs`.

## 2. Scaffold Deliverables
- Created `code/aiosh-rust/aiosh-core/src/repo_health_service.rs` with typed signatures:
  - `pub fn check_git_working_tree(repo_root: &Path) -> RepoHealthCheck`
  - `pub fn check_file_bounds(repo_root: &Path, max_bytes: u64) -> RepoHealthCheck`
  - `pub fn check_security_governance(repo_root: &Path) -> RepoHealthCheck`
  - `pub fn check_repo_health(repo_root: &Path) -> Result<RepoHealthReport, String>`
- Exported `pub mod repo_health_service;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## 3. Compilation Verification Output
```text
    Checking aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 27s
```
