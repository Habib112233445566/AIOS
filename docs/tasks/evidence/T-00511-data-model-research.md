# T-00511 — Evidence & Audit Trail / data model: Research

## 1. Goal
Establish facts, constraints, cryptographic requirements, and prior art for the data model of Evidence & Audit Trail in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase & Architecture):
1. **Audit Ring Hash-Chaining**: The SQLite WAL `audit_ring` table maintains an unbroken SHA-256 hash chain starting from `GENESIS_HASH`, verified by `aiosh-core::audit::AuditRing`.
2. **Task State & Ledger System**:
   - `docs/tasks/MASTER_TASK_LEDGER.jsonl` defines 10,000 tasks executed under the 10-step sub-epic protocol.
   - `docs/tasks/COMPLETIONS.jsonl` logs completion events with task ID, completion timestamp, note, and evidence file path.
   - `docs/tasks/EVENTS.jsonl` maintains append-only state transition events.
3. **Evidence Artifacts on Disk**:
   - Every task produces evidence files under `docs/tasks/evidence/` formatted as markdown documents with structured headers and execution logs.
4. **Canonical JSON Standardization**:
   - `aiosh-core::canonical` enforces deterministic key ordering and formatting for cryptographic hashing across all data structures.

### Assumptions:
1. Automated verification tools need a first-class `TaskEvidence` / `EvidenceRecord` data model in Rust to validate the completeness, SHA-256 integrity, and chronological ordering of evidence files.
2. An `EvidenceManifest` combining task metadata with artifact checksums enables instant tamper-detection and attestation across sub-epics.

## 3. Prior Art & Authoritative Sources
- **in-toto Supply Chain Metadata Framework**: Layout and link metadata structures tracking artifact hashes, step definitions, and inspection rules.
- **SLSA Provenance Specification (v1.0)**: Standardized formats for recording build and execution evidence artifacts.
- **ISO/IEC 27037:2012**: Digital evidence collection and integrity preservation standards.
- **ADR-0035 §F-2**: AIOS audit invariants and hash-chained execution requirements.

## 4. Decisions Needed
1. **Module Placement**: Implement the Evidence data model in `code/aiosh-rust/aiosh-core/src/evidence.rs`.
2. **Core Structs**:
   - `EvidenceRecord`: Models individual evidence artifacts (`task_id`, `step`, `path`, `sha256`, `timestamp_utc`, `status`, `summary`).
   - `TaskEvidenceManifest`: Groups all 10 step artifacts for a task or sub-epic.
   - `EvidenceVerificationReport`: Captures verification status, missing files, and hash mismatches.
3. **Hashing & Serialization**: Enforce SHA-256 hex encoding and canonical JSON serialization.

## 5. Next Steps
Advance to Specification (T-00512) to formalize the data structures, JSON schemas, and validation invariants.
