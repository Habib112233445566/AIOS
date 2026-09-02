# T-00459 — Documentation Index Control / configuration: Documentation

## 1. Documentation Scope
This task documents the `DocIndexConfig` model and configuration files in `docs/README.md`.

## 2. Documentation Additions
- **Document**: `docs/README.md`
- **Section**: `## Documentation Index Control (T-00411..T-00500)`
- **Configuration Layer Added**:
  - `DocIndexConfig` schema and field descriptions (`root_dirs`, `include_extensions`, `exclude_patterns`, `enforce_strict_links`).
  - Resolution chain: `--config <path>` -> `AIOS_DOC_INDEX_CONFIG` -> `docs/doc_index_config.json` -> defaults.
- **Evidence Chain**: Extended through `tasks/evidence/T-00458-configuration-hardening.md`.

## 3. Invariant Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
