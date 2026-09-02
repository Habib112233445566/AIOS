# T-00521 — Evidence & Audit Trail / core service: Research

## 1. Goal
Establish facts, constraints, and prior art for the core service of Evidence & Audit Trail in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase & Architecture):
1. **Evidence Files On Disk**:
   - Evidence files live under `docs/tasks/evidence/` with naming format `T-<5-digit-id>-<descriptor>.md`.
   - Files contain markdown headers with task ID, title, status, and verification outputs.
2. **Audit Ring Database**:
   - SQLite WAL database (`audit_ring`) contains cryptographically linked execution events with `actor`, `tool`, `command`, `args_json`, `outcome`, `prev_hash`, and `hash`.
3. **Canonical Hash Algorithm**:
   - SHA-256 (`aiosh_core::canonical::sha256_hex`) computes deterministic hex hashes from byte slices.
4. **Read Bounds**:
   - File reading is capped at 16 MiB per evidence document (`MAX_DOC_BYTES`) to prevent denial-of-service memory spikes.

### Assumptions:
1. The core service (`evidence_service.rs`) must provide automated scanning and SHA-256 computation of evidence files from disk into structured `TaskEvidenceManifest` objects.
2. Verification operations must verify both file existence and cryptographic hash matches without mutating the disk.

## 3. Prior Art & Authoritative Sources
- **NIST SP 800-86**: Guide to Integrating Forensic Techniques into Incident Response.
- **in-toto Supply Chain Verification Service**: Hash generation and verification of attested step materials.
- **Git Tree Hashes & Object Storage**: Content-addressable hashing model for verifying integrity of source artifacts.

## 4. Decisions Needed
1. **Service Placement**: Implement in `code/aiosh-rust/aiosh-core/src/evidence_service.rs`.
2. **Key Functions**:
   - `compute_file_sha256(path: &Path) -> Result<String, String>`
   - `build_evidence_record_from_file(repo_root: &Path, rel_path: &str, task_id: u32, step: EvidenceStep) -> Result<EvidenceRecord, String>`
   - `verify_evidence_manifest(repo_root: &Path, manifest: &TaskEvidenceManifest) -> Result<EvidenceVerificationReport, String>`
3. **Boundaries & Error Envelopes**:
   - Read cap: 16 MiB max.
   - Non-existent files return explicit errors or are flagged in `EvidenceVerificationReport::missing_files`.

## 5. Next Steps
Advance to Specification (T-00522) to define the function signatures, return envelopes, and error handling contracts.
