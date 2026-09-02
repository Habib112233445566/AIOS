# T-00429 — Documentation Index Control / core service: Documentation

## 1. Documentation Scope
This task documents the core service operations for Documentation Index Control in `docs/README.md`.

## 2. Documentation Updates
- **Document**: `docs/README.md`
- **Section**: `## Documentation Index Control (T-00411..T-00500)`
- **New Operations Added**:
  - `build_doc_index_from_paths`: In-tree document parsing and manifest assembly with 16 MiB read ceiling.
  - `validate_doc_links`: Physical link resolution and repository containment verification.
- **Evidence Chain**: Updated to `tasks/evidence/T-00411-data-model-research.md` .. `tasks/evidence/T-00428-core-service-hardening.md`.

## 3. Invariant Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
