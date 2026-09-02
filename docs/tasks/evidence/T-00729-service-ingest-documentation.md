# T-00729 — Secrets & Access Hygiene / core service: Documentation

## 1. Documentation Scope
This task documents the secrets scanning service in `docs/README.md`.

## 2. Documentation Updates
- Updated `## Secrets & Access Hygiene (T-00711..T-00810)` in `docs/README.md`:
  - Documented `Core Service Operations` (`scan_file_for_secrets`, `scan_workspace_for_secrets`, rule IDs `SEC-001..SEC-005`).
  - Documented automated test runner output covering criteria `K1..K4`.
  - Updated unbroken evidence link chain from `T-00711-data-model-research.md` through `T-00729-service-ingest-documentation.md`.

## 3. Invariant Verification
- `python tools/check_task_docs.py`: C1..C6 PASS.
