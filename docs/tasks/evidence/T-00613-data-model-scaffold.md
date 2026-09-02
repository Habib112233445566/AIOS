# T-00613 — Repository Health / data model: Scaffold

## 1. Scaffolding Scope
This task creates the module skeleton and interface definitions for the **Repository Health** data model in `code/aiosh-rust/aiosh-core/src/repo_health.rs` and exports the module in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## 2. Scaffolding Deliverables
- **`code/aiosh-rust/aiosh-core/src/repo_health.rs`**:
  - `HealthStatus` enum (`Pass`, `Warn`, `Fail`, `Skip`).
  - `HealthCategory` enum (`GitHygiene`, `FileIntegrity`, `SecurityGovernance`, `DependencyHygiene`, `WorkspaceBounds`).
  - `RepoHealthCheck` struct with initial field definitions and validation hooks.
  - `RepoHealthReport` struct with aggregate summary fields and report constructor.
- **`code/aiosh-rust/aiosh-core/src/lib.rs`**:
  - Added `pub mod repo_health;`.

## 3. Build & Compilation Verification
```text
Checking aiosh-core v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) with 0 errors.
```
