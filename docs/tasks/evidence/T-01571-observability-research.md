# T-01571 — Filesystem Layout observability: Research

## Metadata
- **Task ID:** `T-01571`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / observability
- **Status:** Complete — researched observability architecture, telemetry mechanisms, audit streaming, and metric inspection for Filesystem Layout.
- **Date:** 2026-09-19
- **Depends on:** `T-01570` (Sub-Epic 7 Milestone Closure)
- **Feeds:** `T-01572` (Observability Specification)
- **Artifacts:** `docs/tasks/evidence/T-01571-observability-research.md`, `docs/tasks/evidence/T-01571-research.md`

---

## 1. Current State & Prior Art

### 1.1 Existing Observability Substrates in AIOS
1. **SQLite WAL Audit Trail (ADR-0035)**:
   - Located at `$AIOSH_HOME/audit.db` with table `audit_ring`.
   - Schema: `id`, `ts`, `tool`, `command`, `target`, `outcome`, `outcome_detail`, `prev_hash`, `hash`.
   - Every CLI execution and MCP tool invocation writes exactly one SHA-256 hash-chained row.
   - `aiosh audit tail --json -n <N>` provides structured access to recent audit events.
2. **Standard Result Envelopes**:
   - CLI (`--json`): `{"code": <int>, "data": <obj>|null, "error": null|{"code": <str>, "message": <str>}}`.
   - MCP (JSON-RPC 2.0): `{"ok": bool, "audit_id": <int>, "data"|"result": ..., "error": ...}`.
3. **Existing Filesystem Layout Telemetry**:
   - Mutating tools record `target` as layout ID on success and body refusal.
   - `set_active` and `diff` report `destructive_transition` boolean flag.
   - Validation failures carry deterministic error codes (`VALIDATION_FAILED`, `SPEC_PARSE_FAILED`).

---

## 2. Facts vs. Assumptions

| Item | Status | Details |
|---|---|---|
| Audit emission on all calls | **Fact** | ADR-0035 guarantees 100% emission rate for CLI and MCP invocations, including pre-gate refusals and fail-open paths. |
| Hash-chain tamper detection | **Fact** | Rows are cryptographically chained via SHA-256; breaks in the chain indicate tampering. |
| In-band status telemetry | **Fact** | Layout metadata (`active_layout`, partition counts, capacity slack) is available in memory and persisted in `FilesystemLayoutStore`. |
| Dedicated observability tool | **Assumption** | An automated observability suite/tool can verify telemetry consistency across CLI and MCP surfaces without modifying database schemas. |

---

## 3. Decisions & Open Questions

- **Decision 1**: Observability for Filesystem Layout will build directly on existing SQLite WAL audit streams and store metadata rather than adding external daemon dependencies (e.g. Prometheus/OTel collector).
- **Decision 2**: The observability test suite (Criterion `FL12`) will assert:
  - Telemetry fidelity: audit event generation for all 10 MCP tools and 11 CLI subcommands.
  - Latency / timing tracking: duration bounds and structured status envelopes.
  - Queryability: filtering layout audit rows by subsystem `tool LIKE 'aios.fs_layout%'` or `tool = 'fs_layout'`.

---

## 4. Acceptance Confirmation

- [x] Evidence file exists and separates facts from assumptions.
- [x] No code changed; decisions needed are listed explicitly.
