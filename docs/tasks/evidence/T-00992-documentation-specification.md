# T-00992 — Agent Handoff Protocol / Documentation: Specification

## 1. Documentation Structure Specification
- Subsystem Title: `### 8.9 Agent Handoff Protocol (T-00911..T-01000)` (or dedicated section under Phase 0).
- Contents:
  - Architecture overview & data structures (`HandoffRecord`, `HandoffStatus`, `HandoffPriority`, `HandoffReport`).
  - Core service operations (`HandoffStore`, lifecycle state transitions, deduplication, atomic file storage, corruption recovery).
  - CLI usage reference (`aiosh handoff list/show/initiate/accept/reject/complete/cancel`).
  - MCP tool endpoints reference (`aios.handoff.*` JSON-RPC).
  - Configuration guide (`HandoffConfig`, `docs/handoff_config.json`, env vars).
  - Security policy & authorization matrix (`verify_handoff_authorization`).
  - Observability metrics & report validation.
  - Test runner matrix output (`python tools/test_handoff_suites.py` criteria H1..H8).
  - Evidence chain link `tasks/evidence/T-00911-data-model-research.md` .. `tasks/evidence/T-00999-documentation-documentation.md`.
