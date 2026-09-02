# T-00559 — Evidence & Audit Trail / configuration: Documentation

## 1. Documentation Scope
This task documents the `EvidenceConfig` configuration data model, the repository configuration file `config/evidence.config.json`, environmental precedence, and 64 KiB config size limits in `docs/README.md`.

## 2. Documentation Contents
- Documented `EvidenceConfig` settings and default values.
- Documented precedence order (`AIOS_EVIDENCE_CONFIG_PATH` > env vars > `config/evidence.config.json` > in-memory defaults).
- Documented 64 KiB config size limit.
- Updated evidence range to `T-00511`..`T-00558`.

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6).
