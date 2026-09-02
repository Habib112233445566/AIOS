# T-00529 — Evidence & Audit Trail / core service: Documentation

## 1. Documentation Scope
This task documents the core service operations, CLI subcommands, MCP tool endpoints, and security policies for Evidence & Audit Trail in `docs/README.md`.

## 2. Documentation Updates
- Updated `docs/README.md` with:
  - **Core Service Operations**: SHA-256 computation (`compute_file_sha256`), evidence record creation (`build_evidence_record`), manifest verification (`verify_evidence_manifest`).
  - **CLI Surface**: `aiosh evidence verify [--json]`, `aiosh evidence hash <path> [--json]`.
  - **MCP Surface**: `aios.evidence.verify`, `aios.evidence.hash`.
  - **Security Policy**: Read-only vs. PEP-gated mutating actions, 16 MiB size caps, path containment.
  - Updated evidence pointer range (`T-00511`..`T-00528`).

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6).
