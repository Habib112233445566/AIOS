# T-00549 — Evidence & Audit Trail / MCP/API surface: Documentation

## 1. Documentation Scope
This task documents the Model Context Protocol (MCP) tool endpoints for Evidence & Audit Trail (`aios.evidence.verify`, `aios.evidence.hash`, `aios.evidence.scan`) in `docs/README.md`.

## 2. Documentation Updates
- Added `aios.evidence.scan` tool description and JSON-RPC payload example.
- Documented PEP policy (read-only execution vs. mutating token requirement).
- Updated evidence range to `T-00511`..`T-00548`.

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6).
