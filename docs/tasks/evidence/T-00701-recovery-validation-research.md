# T-00701 — Repository Health / recovery & validation: Research

## 1. Goal
Establish facts, failure/drift scenarios, automated validation mechanisms, and recovery strategies for Repository Health in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Codebase & Invariants):
1. **Failure Modes & Drift Scenarios**:
   - Missing, malformed, or oversized (>64 KiB) repository health configuration files (`config/repo_health_config.json` or `docs/repo_health_config.json`).
   - Git working tree degradation (untracked modifications, dirty states, or missing `.git` directories in non-git containers).
   - Workspace file bounds violations (oversized files >16 MiB or unauthorized binary additions).
   - Security governance drift (missing, truncated <100 bytes, or placeholder-filled `SECURITY.md`).
   - Non-zero subprocess exit codes or execution timeouts during git diagnostics.
2. **Validation Invariants**:
   - `RepoHealthReport::validate` enforces arithmetic consistency across `total_checks == passed + warn + failed + skipped`.
   - `RepoHealthCheck::validate` enforces identifier alphanumeric rules, name length bounds (1..128), message size limits (<=1024), and detail limits (<=100 items of <=512 bytes).
   - `tools/test_repo_health_suites.py` enforces criteria H1..H7 across all health layers.
3. **Recovery Invariants**:
   - Resilient Configuration Recovery: `RepoHealthConfig::from_env` and `from_path` safely fall back to canonical in-memory defaults when files are unreadable or missing.
   - Live Report Reconstruction & Reconciliation: `check_repo_health` dynamically re-evaluates all subsystem criteria from live disk state, producing fresh self-consistent telemetry without relying on stale cached reports.

### Assumptions:
1. Automated fallback and reconciliation should provide deterministic zero-downtime health evaluation even when configuration files are missing or damaged.
2. Recovery routines must never perform destructive disk mutations (such as deleting oversized files or discarding uncommitted git changes) without explicit operator authorization.

## 3. Prior Art & Authoritative Standards
- **NIST SP 800-218 (SSDF Tasks PO.3 & PW.2)**: Automated integrity verification and continuous security compliance scanning.
- **OpenSSF Scorecard**: Automated repository health and supply-chain security evaluation.
- **Twelve-Factor App §IX**: Fast startup and resilient fallback to compile-time configuration defaults.

## 4. Decisions Needed
1. **Recovery Helper Standardization**: Implement `recover_default_repo_health_config() -> RepoHealthConfig` and `reconstruct_repo_health_report(repo_root: &Path, config: &RepoHealthConfig) -> Result<RepoHealthReport, String>` in `aiosh-core::repo_health_service`.
2. **Catalog Validation**: Provide `validate_repo_health_report(report: &RepoHealthReport) -> Result<(), String>` and `reconcile_repo_health(repo_root: &Path) -> Result<RepoHealthReport, String>`.

## 5. Next Steps
Advance to Specification (**T-00702**) to formalize the recovery APIs and validation contracts.
