# T-00562 — Evidence & Audit Trail / automated tests: Specification

## 1. Specification Overview
This specification formalizes the automated test contracts for Evidence & Audit Trail, defining the CI validation script `tools/check_evidence.py` and the Rust end-to-end integration test suite `test_evidence_e2e.rs`.

## 2. CI Evidence Validator (`tools/check_evidence.py`)

### Criteria (E1..E4):
- **E1 (Directory Health)**: `docs/tasks/evidence/` directory exists and contains valid markdown files matching `T-<id>-*.md`.
- **E2 (Ledger Consistency)**: Every completed task recorded in `TASK_STATE.json` has corresponding evidence files on disk.
- **E3 (File Bounds & Non-Empty)**: All evidence files are non-empty, valid UTF-8 text, and strictly <= 16 MiB.
- **E4 (Deterministic Checksums)**: SHA-256 digests computed in Python match standard hex outputs.

### Exit Codes:
- `0`: All checks (E1..E4) pass cleanly.
- `1`: One or more validation checks failed.

## 3. Rust End-to-End Test Contract (`test_evidence_e2e.rs`)
- **Lifecycle Sequence**:
  1. Initialize temp repository workspace.
  2. Create `EvidenceConfig` with custom directory.
  3. Generate mock task evidence files for 10-step sub-epic progression.
  4. Construct `TaskEvidenceManifest` containing 10 `EvidenceRecord` items.
  5. Run `verify_evidence_manifest` and assert `is_valid == true`.
  6. Tamper with one file's content on disk.
  7. Re-run `verify_evidence_manifest` and assert `is_valid == false` with 1 hash mismatch.
  8. Delete one file from disk and assert `missing_files` detection.
