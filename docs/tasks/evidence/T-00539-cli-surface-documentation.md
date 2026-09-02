# T-00539 — Evidence & Audit Trail / CLI surface: Documentation

## 1. Documentation Scope
This task documents the `aiosh evidence` CLI suite (`verify`, `hash`, `scan`), command syntax, options, and example invocations in `docs/README.md`.

## 2. Documentation Contents
- Updated `docs/README.md` with:
  - `aiosh evidence verify [--repo <path>] [--manifest <path>] [--json]`
  - `aiosh evidence hash <path> [--json]`
  - `aiosh evidence scan [--repo <path>] [--task <id>] [--json]`
  - Copy-pasteable example commands in JSON and text modes.
  - Security policy rules (unauthenticated read-only vs PEP-gated mutations).
  - Updated evidence pointer range (`T-00511`..`T-00538`).

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6).
