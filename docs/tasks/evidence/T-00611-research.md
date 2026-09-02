# T-00611 — Repository Health / data model: Research

## 1. Goal
Establish empirical facts, constraints, authoritative standards, and prior art for the data model of the **Repository Health** component (`T-00611..T-00710`) in Phase 0 of AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase & Architecture):
1. **Multi-Substrate Repository Structure**:
   - The workspace contains mixed language substrates: Rust (`code/aiosh-rust`), Python CLI & MCP tools (`code/aiosh-cli`, `code/aiosh-mcp`), and administrative/verification tools (`tools/`).
2. **Existing Invariant Checkers**:
   - `tools/check_security_policy.py`: Enforces presence and structure of `SECURITY.md`, absence of TODOs, verbatim advisory URL, and valid referenced paths.
   - `tools/check_task_docs.py`: Enforces documentation structure, link validities, phase mapping, and non-volatile count invariants.
   - `tools/check_evidence.py`: Enforces evidence directory health, ledger consistency, file bounds (non-empty UTF-8), and deterministic SHA-256 digests.
3. **Canonical Data Model Pattern in `aiosh-core`**:
   - All Phase 0 components define their fundamental data structures in `code/aiosh-rust/aiosh-core/src/<component>.rs` using `serde::{Serialize, Deserialize}`, strict runtime validation, and canonical JSON serialization.

### Assumptions:
1. Operators and automated agents require a unified, structured assessment of overall repository health spanning working tree status, file bounds, line ending hygiene, dependency lockfiles, and security policies.
2. A structured report model (`RepoHealthReport`) consisting of granular check items (`RepoHealthCheck`) with discrete severity levels (`Pass`, `Warn`, `Fail`, `Skip`) and domain categories (`GitHygiene`, `FileIntegrity`, `SecurityGovernance`, `DependencyHygiene`, `WorkspaceBounds`) provides optimal diagnostic granularity.

## 3. Prior Art & Authoritative Sources
- **OpenSSF Scorecard v5**: Standard automated security metric definitions (Branch-Protection, Security-Policy, Dependency-Update-Tool, Dangerous-Workflows, Pinned-Dependencies).
- **OpenSSF SCM Best Practices Working Group**: Guidance for secure repository layout, access control, and hygiene.
- **Git SCM Architecture (v2.4x)**: Working tree states (clean, dirty, untracked, ignored), index invariants, and submodule tracking.
- **POSIX.1-2017 (IEEE Std 1003.1)**: Standard file system semantics, path length limits (`PATH_MAX`), and file permission modes.
- **Twelve-Factor App §I & §X**: Single codebase tracked in revision control; dev/prod parity and configuration isolation.

## 4. Decisions Needed
1. **Module Placement**:
   - Data structures placed in `code/aiosh-rust/aiosh-core/src/repo_health.rs`.
2. **Core Structs & Enums**:
   - `HealthStatus`: `Pass`, `Warn`, `Fail`, `Skip`.
   - `HealthCategory`: `GitHygiene`, `FileIntegrity`, `SecurityGovernance`, `DependencyHygiene`, `WorkspaceBounds`.
   - `RepoHealthCheck`: `check_id`, `name`, `category`, `status`, `message`, `details`, `duration_ms`.
   - `RepoHealthReport`: `repo_path`, `timestamp_utc`, `overall_status`, `total_checks`, `passed_checks`, `warn_checks`, `failed_checks`, `checks`, `metadata`.
3. **Validation Rules**:
   - Non-empty `repo_path` and `timestamp_utc`.
   - Aggregate check counts must match length of `checks` vector.
   - Strict bounds on strings and message lengths.

## 5. Next Steps
Advance to Specification (`T-00612`) to formalize JSON schemas, field-level validation contracts, and error conditions.
