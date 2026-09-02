# T-00672 — Repository Health / security policy: Specification

## Contract
- **Function**: `check_security_governance(repo_root: &Path) -> RepoHealthCheck`
- **Inputs**: Repository root path.
- **Outputs**: `RepoHealthCheck` with `check_id = "security_governance"`, `category = SecurityGovernance`.
- **Pass**: SECURITY.md exists, ≥100 chars, no TODO markers.
- **Fail**: Missing file, too short, or contains TODO.
- **Warn**: File read failure.
