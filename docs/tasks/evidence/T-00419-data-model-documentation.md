# T-00419 — Documentation Index Control / data model: Documentation

## 1. Documentation Scope
This task documents the Documentation Index Control data model in `docs/README.md` under `## Documentation Index Control (T-00411..T-00500)`.

## 2. Documentation Additions
- **Section**: `## Documentation Index Control (T-00411..T-00500)` in `docs/README.md`.
- **Data Model Overview**: Documents `DocIndexEntry` and `DocIndexManifest` types and helper methods.
- **Copy-Pasteable Example**: Adds a valid JSON-RPC schema payload representing a documentation index manifest.
- **Hardening Limits**: Documents the 10,000 entry cap and 1,000 links-per-entry cap.
- **Evidence Chain**: Links `tasks/evidence/T-00411-data-model-research.md` .. `tasks/evidence/T-00418-data-model-hardening.md`.

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
