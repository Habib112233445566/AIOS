# T-01572 — Filesystem Layout observability: Specification

## Metadata
- **Task ID:** `T-01572`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / observability
- **Status:** Complete — defined observability contract, telemetry schemas, audit queryability, and status inspection.
- **Date:** 2026-09-19
- **Depends on:** `T-01571` (Research)
- **Feeds:** `T-01573` (Scaffold)
- **Artifacts:** `docs/tasks/evidence/T-01572-observability-specification.md`, `docs/tasks/evidence/T-01572-spec.md`

---

## 1. Observability Contract Specification

### 1.1 Telemetry & Audit Stream Schema (ADR-0035)
Every invocation of the filesystem layout subsystem across CLI and MCP emits a structured event into the SQLite WAL audit ring:

| Field | Type | Description |
|---|---|---|
| `id` | `INTEGER PRIMARY KEY` | Sequential event identifier. |
| `ts` | `TEXT` | RFC 3339 UTC timestamp (e.g. `2026-09-19T05:40:00Z`). |
| `tool` | `TEXT` | Subsystem identifier: `"fs_layout"` (CLI) or `"aios.fs_layout.<verb>"` (MCP). |
| `command` | `TEXT` | Subcommand/verb name (`"validate"`, `"register"`, `"set_active"`, `"remove"`, etc.). |
| `target` | `TEXT` / `NULL` | Layout ID targeted by the operation (or `NULL` on pre-gate refusal). |
| `outcome` | `TEXT` | `"ok"` / `"success"` (completed), `"error"` (failed), `"refused"` (policy/PEP refusal). |
| `outcome_detail` | `TEXT` | Explanatory message, error code, or transition metadata (e.g. `"destructive: true"`). |
| `prev_hash` | `TEXT` | SHA-256 hash of preceding row. |
| `hash` | `TEXT` | SHA-256 hash of current row content chained to `prev_hash`. |

---

### 1.2 Observability Test Criteria (FL12)

The observability test suite must verify the following five criteria (`O1`..`O5`):
- **O1: Telemetry Emission Completeness**: Every invocation (read-only and mutating, CLI and MCP) produces an audit row.
- **O2: Audit Correlation & Queryability**: Events can be queried and filtered by `tool` and `target` using `aiosh audit tail`.
- **O3: Outcome Fidelity**: Successes log `"ok"`/`"success"`, errors log `"error"`, and policy denials log `"refused"`.
- **O4: State Inspection Parity**: In-band status telemetry (`active_layout`, partition counts) is consistently exposed on both CLI and MCP.
- **O5: Destructive Mutation Flagging**: Destructive state transitions (partition shrink/delete) explicitly expose `destructive: true` in the output envelope.

---

## 2. Reused vs New Interfaces

- **Reused**:
  - Existing `audit_ring` SQLite table and `aiosh audit tail` command.
  - Existing MCP tool and CLI command envelopes.
- **New**:
  - `code/aiosh-cli/tests/test_fs_layout_observability.py`: Automated test suite exercising O1..O5.
  - Criterion `FL12` registration in `tools/test_fs_layout_suites.py`.

---

## 3. Acceptance Confirmation

- [x] Spec covers happy path, failure path, and audit effects.
- [x] Spec is reviewable without reading the implementation.
