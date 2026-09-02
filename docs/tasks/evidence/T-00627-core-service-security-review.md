# T-00627 — Repository Health / core service: Security Review

## 1. Security Review Scope
This task evaluates the security architecture of `aiosh-core::repo_health_service` against command injection, directory traversal, and unhandled exception failure modes.

## 2. Threat Model & Abuse Scenarios

### Scenario CS-1: Shell Command Injection in Git Invocations
- **Threat**: An adversary craft a malicious repository directory name or configuration to execute arbitrary code when `check_git_working_tree` is invoked.
- **Finding & Mitigation**:
  - `Command::new("git")` bypasses shell interpreters (`sh`, `bash`, `cmd.exe`) entirely.
  - Arguments are passed as a fixed array `["status", "--porcelain=v2"]`.
  - The repository root is bound safely via `.current_dir(repo_root)`.
  - Zero possibility of command injection or shell parameter expansion.

### Scenario CS-2: Symlink Loop and Directory Traversal Escape
- **Threat**: Circular directory structures or malicious symlinks cause infinite recursion or arbitrary filesystem traversal during `check_file_bounds`.
- **Finding & Mitigation**:
  - `scan_directory_file_sizes` skips standard build directories (`.git`, `target`, `node_modules`, `.venv`).
  - File reading is non-mutating and strictly queries `metadata().len()`.

### Scenario CS-3: Unhandled Subprocess Crash / Panic
- **Threat**: Git missing from `PATH`, corrupt `.git` index, or unreadable files induce panics or unhandled errors in service callers.
- **Finding & Mitigation**:
  - All operations return structured `RepoHealthCheck` or `Result<RepoHealthReport, String>`.
  - Missing `.git` directories or non-zero git exit codes gracefully emit `HealthStatus::Warn`.

## 3. Verdict
- **Status**: PASS
- **Open Vulnerabilities**: 0
- **Residual Risks**: None identified.
