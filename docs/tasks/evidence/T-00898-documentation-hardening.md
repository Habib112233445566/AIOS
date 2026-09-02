# T-00898 — Regression Triage / Documentation: Hardening

## 1. Hardening Deliverables
- **Documentation Rot-Proofing**:
  - `tools/check_task_docs.py` enforces C1..C6 structural checks in CI.
  - Required exact in-tree resolution for all cross-linked files and evidence paths.
- **Fail-Fast Error Diagnostics**:
  - Documentation checker outputs exact mismatch line numbers and non-zero exit code upon invariant violation.
