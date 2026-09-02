# T-00519 — Evidence & Audit Trail / data model: Documentation

## 1. Documentation Scope
This task documents the core data models, validation constraints, and serialization rules for Evidence & Audit Trail in `docs/README.md`.

## 2. Documentation Updates
- Added **Evidence & Audit Trail** section to `docs/README.md`:
  - `EvidenceStep` enum (10 sub-epic lifecycle steps).
  - `EvidenceRecord` struct and fields.
  - `TaskEvidenceManifest` aggregation and query helpers.
  - `EvidenceVerificationReport` validation summary.
  - Invariants: 1..10000 task ID bounds, relative non-escaping paths, 64-char lowercase SHA-256 hex checksums, 10,000 record capacity cap.
- Added evidence pointer range (`T-00511`..`T-00518`).

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6).
