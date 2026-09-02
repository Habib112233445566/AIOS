# T-00791 — Secrets & Access Hygiene / documentation: Research

## 1. Prior Art & Documentation Infrastructure
- **Central Reference**: `docs/README.md` under `## Secrets & Access Hygiene (T-00711..T-00810)`.
- **Documentation Invariant Checker (`tools/check_task_docs.py`)**:
  - C1: Spec health and markers.
  - C2: Component sections.
  - C3: Referenced file paths exist in tree.
  - C4: Phase map consistency.
  - C5: Index health and link boundaries.
  - C6: No volatile count snapshots.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Invariant Compliance | Fact | `tools/check_task_docs.py` enforces rot-proof documentation. |
| Surface Coverage | Fact | Documentation must comprehensively cover Rust data structures, CLI subcommands, MCP tool schemas, JSON configuration, and security policy. |
| Reproducible Examples | Fact | All commands and JSON payloads in documentation must be runnable and syntactically valid. |

## 3. Decisions & Contracts Needed
1. Structure the comprehensive documentation section in `docs/README.md` with explicit subsections for Architecture, CLI Usage, MCP Tool Specifications, Configuration Schema, Automated Testing, Security Invariants, and Observability.
2. Link evidence chain `T-00711` through `T-00799` in `docs/README.md`.
