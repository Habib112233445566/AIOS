# T-00891 — Regression Triage / Documentation: Research

## 1. Prior Art & Documentation Infrastructure
- **Central Reference**: `docs/README.md` under `## Regression Triage (T-00811..T-00910)`.
- **Documentation Invariant Checker (`tools/check_task_docs.py`)**:
  - `C1`: Spec health and absence of unfulfilled markers.
  - `C2`: Component sections.
  - `C3`: Referenced in-tree paths exist.
  - `C4`: Phase map consistency.
  - `C5`: Index health and link boundaries.
  - `C6`: Absence of volatile count snapshots.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Invariant Compliance | Fact | `tools/check_task_docs.py` enforces rot-proof documentation. |
| Surface Coverage | Fact | Documentation covers data structures, CLI subcommands, MCP tool schemas, configuration, automated tests, security policy, and observability. |
| Reproducible Examples | Fact | All commands and JSON payloads in documentation are verified and syntactically valid. |

## 3. Decisions & Actions
- Ensure `docs/README.md` comprehensively documents all 7 shipped subcomponents for Regression Triage and links task evidence files.
