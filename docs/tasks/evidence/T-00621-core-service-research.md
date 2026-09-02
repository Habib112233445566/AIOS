# T-00621 — Repository Health / core service: Research

## 1. Problem Statement & Background
The `repo_health_service` provides automated evaluation of repository working tree hygiene, file bounds, line ending consistency, security governance artifacts, and workspace boundaries across all AIOS substrates.

## 2. Authoritative Sources & Prior Art

### A. Git SCM Porcelain v2 Specification
- **Citation**: Git Documentation, `git-status(1)` [https://git-scm.com/docs/git-status](https://git-scm.com/docs/git-status)
- **Facts**:
  - `git status --porcelain=v2` provides machine-stable, parseable output ignoring user-configured terminal colors or formatting.
  - Headers are prefixed with `# ` (e.g. `# branch.oid <commit>`, `# branch.head <branch>`, `# branch.upstream <upstream>`).
  - Tracked file changes are prefixed with `1 <XY> ...` (ordinary) or `2 <XY> ...` (renamed/copied).
  - Untracked files are prefixed with `? <path>`, and ignored files with `! <path>`.
- **Engineering Implications**:
  - The health service can determine if working tree is clean by checking for non-zero change entries (`1`, `2`, `?`, `u`).

### B. OpenSSF Scorecard v5 (Repository Security Best Practices)
- **Citation**: OpenSSF Scorecard Architecture & Heuristics [https://github.com/ossf/scorecard](https://github.com/ossf/scorecard)
- **Facts**:
  - OpenSSF evaluates repository health against binary artifacts, security policies (`SECURITY.md`), dependency hygiene, and branch protection.
  - Binary Artifact check: flags unreviewable executable binaries, `.exe`, `.so`, `.dylib`, `.jar` stored directly in git trees without explicit exemption.
- **Engineering Implications**:
  - `repo_health_service` should include automated scans for forbidden binary extensions in source directories.

### C. POSIX.1-2017 & Cross-Platform Path Invariants
- **Citation**: IEEE Std 1003.1-2017 (POSIX.1-2017)
- **Facts**:
  - Canonical repo-relative paths use forward slashes (`/`), avoid leading/trailing slashes, and prohibit traversal sequences (`..`).
  - Line ending invariants require LF (`\n`) for text files, flagging CRLF or mixed line endings in repository source code.

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Working Assumption |
| :--- | :--- | :--- |
| **Git Invocation** | `git status --porcelain=v2` exits 0 on valid repos, non-zero if not a git directory. | The host environment has `git` available in `PATH` or returns a graceful `Skip`/`Warn` status if missing. |
| **File Scans** | Direct directory walking with `std::fs` allows deterministic, sub-second inspection. | All monitored source files reside within the repository root bounds. |
| **Report Structure** | `RepoHealthReport` validates `total_checks == passed + warn + failed + skipped`. | Individual checks execute concurrently or sequentially with independent duration tracking. |

## 4. Key Design Decisions for Core Service
1. **Service Module**: Implement `aiosh-core::repo_health_service` containing `check_repo_health(&repo_root: &Path) -> Result<RepoHealthReport, String>`.
2. **Built-in Checks**:
   - `git_working_tree`: Verifies clean git working tree via porcelain v2 parsing.
   - `file_bounds`: Verifies no individual file exceeds the 16 MiB maximum size cap.
   - `security_policy`: Verifies root `SECURITY.md` presence and validity.
   - `forbidden_binaries`: Verifies absence of unapproved binary installer executables (`.exe`, `.bin`) in source trees.
3. **Execution Safety**:
   - Subprocess calls are isolated and bounded by a 5-second timeout.
   - Errors fail closed into structured `HealthStatus::Fail` or `HealthStatus::Warn` checks rather than panicking.
