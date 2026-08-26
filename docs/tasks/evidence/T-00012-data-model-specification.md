# T-00012 — Task Ledger Control: data model specification

**Type:** specification (no code changed)
**Date:** 2026-08-21
**Depends on:** T-00011 research (`T-00011-data-model-research.md`)
**Resolves:** decisions D1–D6 from the research note.

---

## 1. Components and roles

| Component | Kind | Mutability | Role |
|---|---|---|---|
| `docs/tasks/MASTER_TASK_LEDGER.jsonl` | JSONL, 10,000 lines | **Immutable** after generation (regenerate wholesale only) | Plan of record: every task's goal, instructions, acceptance, artifacts |
| `docs/tasks/COMPLETIONS.jsonl` | JSONL, append-only | **Append-only** (never rewritten) | Event log of every completion/block/unblock (NEW, decision D4) |
| `docs/tasks/TASK_STATE.json` | JSON object | Rewritten **atomically** (tmp + `os.replace`) | Derived live pointer: `next_task`, `completed[]`, `blocked[]` (D1, D2) |
| `docs/tasks/evidence/T-NNNNN*.md` | Markdown | Append/new files only | Per-task proof of acceptance |
| `tools/complete_task.py` | Script | n/a | The only sanctioned state mutator |
| `tools/generate_master_tasks.py` | Script | n/a | Deterministic ledger generator |

**Single source of truth for status:** `TASK_STATE.json` (decision D2).
The ledger's per-task `status` field is generation-time metadata only and
is never updated; agents and tools MUST NOT read it for scheduling.

## 2. Schemas

### 2.1 Ledger record (one JSONL line) — unchanged, pinned

```json
{
  "id": 12,
  "title": "<string>",
  "phase": "<string>",
  "status": "pending|ready|done (generation-time only, NOT authoritative)",
  "goal": "<string>",
  "instructions": ["<string>", ...],
  "acceptance": ["<string>", ...],
  "artifacts": ["<path>", ...],
  "depends_on": [11],
  "next_task": 13
}
```

Invariants (enforced by generator, asserted in CI):
- ids are exactly 1..10000, contiguous, unique
- `depends_on == [id - 1]` for id > 1, `[]` for id == 1
- `next_task == id + 1`, or `null` for id == 10000
- custom tasks 1–10 occupy Phase 0 slots 1–10 verbatim

### 2.2 Completion event record (NEW — `COMPLETIONS.jsonl`)

```json
{"seq": 8, "ts": "2026-08-21T07:00:00Z", "event": "completed",
 "task_id": 11, "note": "<free text>", "evidence": ["docs/tasks/evidence/T-00011-data-model-research.md"]}
```

- `event` ∈ `completed | blocked | unblocked | pointer_reset`
- `seq` is monotonically increasing, `seq == previous seq + 1`, first is 1
- `completed`/`blocked` require `task_id == TASK_STATE.next_task` at append time
- `unblocked`/`pointer_reset` are human-override events; they carry a
  mandatory non-empty `note` and set the next pointer explicitly
  (decision D5: skipping a blocked task is NEVER silent — it is an
  explicit event in the log)

### 2.3 State pointer (TASK_STATE.json) — v2

```json
{
  "schema_version": 2,
  "ledger": "docs/tasks/MASTER_TASK_LEDGER.jsonl",
  "total_tasks": 10000,
  "next_task": 12,
  "completed": [1, 2, 11],
  "blocked": [],
  "last_completed_at": "2026-08-21T06:38:06Z",
  "last_event_seq": 8,
  "rule": "Execute ONLY next_task. Advance by exactly 1 via tools/complete_task.py."
}
```

Migration: a v1 file (no `schema_version`, no `blocked`, no
`last_event_seq`) is accepted on read; the next write upgrades it to v2
and seeds `last_event_seq` from the COMPLETIONS.jsonl line count.

## 3. Interfaces

### Reused (existing, unchanged)

- `tools/complete_task.py <id> [--note ...]` — CLI contract kept.
- `tools/generate_master_tasks.py` — regeneration entry point.
- JSON Lines conventions per jsonlines.org; POSIX `rename(2)` atomicity
  (same pattern as `retention.py` archive writes).
- Repo docs referencing the pointer (README.md, docs/README.md, GOALS.md).

### New

- `COMPLETIONS.jsonl` event log (D4).
- `complete_task.py rebuild` — recompute `TASK_STATE.json` from
  `COMPLETIONS.jsonl` (Fowler "Complete Rebuild"; the pointer is derived
  state).
- Exclusive advisory lock `docs/tasks/.TASK_STATE.lock` via `fcntl.flock`
  around read-check-write (D3, cheap insurance; single-writer remains
  the documented operating assumption).

## 4. `complete_task.py <id>` — behavior contract

Happy path:
1. Acquire exclusive flock on `.TASK_STATE.lock`.
2. Read `TASK_STATE.json`; reject if `id != next_task` (exit 1, JSON
   error `NO-SKIP violation`, zero state change).
3. Verify `id` exists in the ledger JSONL (stream-scan by `"id": <n>,`).
4. Append one `completed` event to `COMPLETIONS.jsonl` (single line,
   `flush` + `os.fsync`).
5. Build new state (`completed += [id]`, `next_task = id + 1`,
   `last_completed_at = now`, `last_event_seq = seq`); write to
   `TASK_STATE.json.tmp.<pid>`; `os.replace` onto `TASK_STATE.json`.
6. Ensure evidence stub `T-<id:05d>-completion.md` exists (acceptance
   checkboxes from the ledger).
7. Print `{ok: true, completed: id, next_task: id+1, evidence: <path>}`;
   exit 0.

Failure paths:

| Condition | Result | State effect |
|---|---|---|
| `id != next_task` | exit 1, `NO-SKIP violation` JSON | none |
| `id` not in ledger | exit 1, `unknown task` | none |
| state file missing/corrupt | exit 2, hint: run `complete_task.py rebuild` | none |
| ledger line unparseable | exit 2, report line number | none |
| crash between steps 4 and 5 | event logged, pointer stale | `rebuild` repairs (event log is authoritative) |
| crash between steps 5 and 6 | pointer advanced, stub missing | cosmetic; next completion unaffected |

Audit effects: every state transition appends exactly one event line;
events are never rewritten or deleted (mirrors Constitution P-2
philosophy applied to the task ledger).

## 5. Blocking (D5)

- `complete_task.py block <id> --reason "..."` (only when `id == next_task`):
  appends a `blocked` event, adds `id` to `blocked[]`, pointer does NOT
  advance. Work halts.
- Resuming requires human override:
  `complete_task.py unblock <id> --reason "..."` (appends `unblocked`,
  removes from `blocked[]`, pointer unchanged) to retry, or
  `complete_task.py skip <id> --reason "..."` (appends `pointer_reset`
  with mandatory reason, sets `next_task = id + 1`, records `id` in a
  `skipped[]` list). Skipping is always visible in the event log.

## 6. Determinism & reproducibility (C-2)

`generate_master_tasks.py` is a pure function of its embedded phase/epic
tables: running it twice yields byte-identical ledger output. Regeneration
MUST NOT change ids of already-completed tasks; any restructure requires a
`pointer_reset` event documenting the mapping.

## 7. Legacy disposition (D6)

`mostimportanAIfolder/TASK_DATABASE.json` stays frozen in place with its
`authoritative: false` marker. No reads for scheduling. README/GOALS
already forbid using it to pick work.

## 8. Out of scope (deferred)

- Concurrent multi-agent writers (flock covers same-host accidents; true
  multi-agent needs a real lock service — revisit if A1 breaks).
- Machine validation of evidence content against acceptance criteria
  (evidence remains human/agent-attested).
- Ledger sharding into per-phase files (10k × ~0.6 KB ≈ 6 MB is fine for
  text tooling; revisit only if tooling chokes).
