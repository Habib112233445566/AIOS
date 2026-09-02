# T-00991 — Agent Handoff Protocol / Documentation: Research

## 1. Documentation Scope & Invariant Mapping
- Agent Handoff Protocol Epic (`T-00911..T-01000` — 100 consecutive tasks) encompasses:
  1. Data Model (`HandoffRecord`, `HandoffStatus`, `HandoffPriority`, `HandoffReport`, SHA-256 signatures).
  2. Core Service (`HandoffStore`, lifecycle transitions, deduplication, atomic persistence, corruption recovery).
  3. CLI Surface (`aiosh handoff [list|show|initiate|accept|reject|complete|cancel]` with audit rows).
  4. MCP/API Surface (`aios.handoff.*` tools, PEP gating, SQLite WAL audit logging).
  5. Configuration (`HandoffConfig`, storage caps, TTL timeouts, env vars, `docs/handoff_config.json`).
  6. Automated Tests (State transition matrix, edge case fuzzing, 50+ batch load).
  7. Security Policy (`verify_handoff_authorization`, role-based caller isolation).
  8. Observability (`HandoffReport` distribution, metrics validation).
  9. Documentation & Final Verification (Full rot-proof doc sync and whole-epic closure).
- Checked against rot-proof documentation invariants C1..C6 in `tools/check_task_docs.py`.
