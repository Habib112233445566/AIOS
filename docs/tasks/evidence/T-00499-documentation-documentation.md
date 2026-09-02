# T-00499 — Documentation Index Control / documentation: Documentation

## 1. Documentation Scope
This task verifies that user, operator, and agent documentation for Documentation Index Control in `docs/README.md` is complete, accurate, and includes full interface definitions, config references, and working examples.

## 2. Documentation Contents in `docs/README.md`
- **CLI Commands**: `aiosh doc show`, `aiosh doc check`, `aiosh doc search` with `--json` and `--config` support.
- **MCP Endpoints**: `aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`.
- **Configuration Hierarchy**: `DocIndexConfig` (`root_dirs`, `include_extensions`, `exclude_patterns`, `enforce_strict_links`).
- **Automated Tests**: Suite runners and standalone unit tests.
- **Security Policy & PEP**: Grant gating, size limits, repo root confinement.
- **Observability**: `DocIndexTelemetry` model and structured diagnostic logging.
- **Evidence Chain**: `T-00411`..`T-00498`.

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6).
