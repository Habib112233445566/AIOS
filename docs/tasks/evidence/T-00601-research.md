# T-00601 — Evidence & Audit Trail / recovery & validation: Research

## 1. Goal
Establish facts, failure/drift scenarios, automated validation mechanisms, and recovery strategies for Evidence & Audit Trail in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Codebase & Invariants):
1. **Drift & Failure Modes**:
   - Missing or deleted evidence markdown files in `docs/tasks/evidence/`.
   - Checksum mismatches resulting from file modifications, disk truncation, or partial writes.
   - Malformed, oversized (>64 KiB), or missing configuration files (`config/evidence.config.json`).
   - Out-of-bounds path traversal attempts (`../../`) aiming to index unauthorized files outside the repository.
   - Files exceeding 16 MiB maximum ingestion bounds.
2. **Validation Invariants**:
   - `verify_evidence_manifest` validates existence and deterministic SHA-256 digests across all recorded artifacts.
   - `tools/check_evidence.py` enforces criteria `E1` (directory health), `E2` (ledger consistency), `E3` (file bounds & valid UTF-8), and `E4` (hash consistency).
3. **Recovery Invariants**:
   - Resilient Configuration Fallback: `EvidenceConfig::from_env` safely falls back to compile-time in-memory defaults when configuration files are missing or unreadable.
   - Live Manifest Reconstruction: `scan_evidence_files` enables reconstructive indexing of on-disk evidence artifacts without relying on compromised or missing manifest JSON files.

### Assumptions:
1. Automated reconstruction from disk allows zero-downtime recovery of task evidence catalogs during CI or disaster recovery workflows.
2. Fallback defaults should never crash CLI or MCP daemons upon filesystem anomalies.

## 3. Prior Art & Authoritative Standards
- **NIST SP 800-218 (SSDF Tasks PW.1 & PW.2)**: Automated software component verification and attestation recovery.
- **SLSA v1.0 (Supply-chain Levels for Software Artifacts)**: Cryptographic artifact verification and attestation reproducibility.
- **Twelve-Factor App §IX (Disposability & Fast Startup)**: Resilient fallback to compile-time configuration defaults.

## 4. Decisions Needed
1. **Recovery Helper Standardization**: Provide a dedicated `recover_evidence_manifest(repo_path: &Path, task_range: &str, epic_name: &str) -> Result<TaskEvidenceManifest, String>` in `aiosh-core::evidence_service`.
2. **Actionable Diagnostics**: Ensure `EvidenceVerificationReport` clearly articulates missing paths and hash mismatches for autonomous agent remediation.

## 5. Next Steps
Advance to Specification (**T-00602**) to formalize the recovery APIs and validation contracts.
