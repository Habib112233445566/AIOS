# T-00439 — Documentation Index Control / CLI surface: Documentation

## 1. Documentation Scope
This task documents the `aiosh doc` CLI commands and usage examples in `docs/README.md`.

## 2. Documentation Additions
- **Document**: `docs/README.md`
- **Section**: `## Documentation Index Control (T-00411..T-00500)`
- **Commands Added**:
  - `aiosh doc show [--json]`
  - `aiosh doc check [--repo <path>] [--json]`
  - `aiosh doc search <query> [--json]`
- **Evidence Chain**: Extended through `tasks/evidence/T-00438-cli-surface-hardening.md`.

## 3. Invariant Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
