# T-01129 — Base Image Build / CLI Surface: Documentation

**Date:** 2026-09-03
**Type:** Documentation
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / CLI Surface

## 1. Documentation Synchronizations
- Documented `aiosh image` CLI subcommands (`list`, `show`, `plan`, `filter`) and arguments in `docs/README.md`.
- Documented security hardening rules (control character prohibition on identifiers).
- Updated standalone test runner snippet to criteria B1..B3.
- Validated all documentation invariants C1..C6 via `tools/check_task_docs.py`.
