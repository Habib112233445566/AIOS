# T-00449 — Documentation Index Control / MCP/API surface: Documentation

## 1. Documentation Scope
This task documents the MCP tools (`aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`) in `docs/README.md`.

## 2. Documentation Additions
- **Document**: `docs/README.md`
- **Section**: `## Documentation Index Control (T-00411..T-00500)`
- **MCP Tools Added**:
  - `aios.doc.index.get`: Returns documentation manifest catalog.
  - `aios.doc.check`: Validates documentation graph links.
  - `aios.doc.search`: Queries indexed documentation entries with keyword filter.
- **Evidence Chain**: Extended through `tasks/evidence/T-00448-mcp-api-surface-hardening.md`.

## 3. Invariant Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
