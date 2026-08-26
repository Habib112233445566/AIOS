# T-00011 — Task Ledger Control: data model research

**Type:** research only (no code changed)
**Date:** 2026-08-21
**Scope:** facts, constraints, and prior art for the data model behind
`docs/tasks/` — the sequential 10,000-task ledger, its live pointer,
and the completion tooling.

---

## 1. Problem

The project replaced a non-authoritative, graph-derived reconstruction
(`mostimportanAIfolder/TASK_DATABASE.json`, which self-declares
`authoritative: false`, `provenance: graph-derived-recovery`) with a
strictly sequential ledger so agents execute task N+1 only after task N.
The data model must support: deterministic ordering, mechanical no-skip
enforcement, crash-safe state updates, agent handoff, and per-task
evidence. This note establishes what exists, what is proven, and what
decisions remain.

## 2. Current implementation (facts, verified in-tree 2026-08-21)

| Artifact | Format | Role | Writer |
|---|---|---|---|
| `docs/tasks/MASTER_TASK_LEDGER.jsonl` | JSON Lines, 10,000 lines, ids 1..10000 | immutable plan of record | `tools/generate_master_tasks.py` (regenerated wholesale) |
| `docs/tasks/TASK_STATE.json` | JSON object | mutable live pointer: `next_task`, `completed[]`, `last_completed_at` | `tools/complete_task.py` |
| `docs/tasks/MASTER_TASK_LEDGER.md` | Markdown | human index (law + phase map + first 25 tasks) | generator |
| `docs/tasks/GOALS.md` | Markdown | mission + no-skip law | manual |
| `docs/tasks/evidence/T-NNNNN*.md` | Markdown | per-task proof of acceptance | agent + completion tool stub |

Ledger record schema (every line, verified by parse):
`id, title, phase, status, goal, instructions[], acceptance[],
artifacts[], depends_on[], next_task`.

Observed invariants:
- `depends_on == [id-1]` and `next_task == id+1` for all generated rows
  (strict linear order; custom tasks 1–10 override Phase-0 slot 1–10).
- `complete_task.py` refuses any `task_id != TASK_STATE.next_task`
  (proven: attempting T-00007 while `next_task=5` returned a NO-SKIP
  violation, exit 1).
- TASK_STATE.json `completed` is append-only in practice; pointer moves
  by exactly +1.

## 3. Authoritative sources / prior art

### 3.1 JSON Lines (ledger file format)

Source: <https://jsonlines.org/> (fetched 2026-08-21).

Established facts:
- One JSON value per line, `'\n'` terminator, UTF-8, no BOM
  (per RFC 8259 §8.1).
- Designed for "storing structured data that may be processed one
  record at a time"; good for logs and inter-process messages.
- `.jsonl` extension is the convention; stream compression
  (`.jsonl.gz`) recommended for space.

Application: MASTER_TASK_LEDGER.jsonl already complies (each line
parses standalone; generator writes with `ensure_ascii=False`,
`separators=(",",":")`). Line-oriented format means a corrupted line
affects exactly one task and is trivially detectable by per-line parse.

### 3.2 Event sourcing (state-pointer design)

Source: Martin Fowler, *Event Sourcing* (12 Dec 2005),
<https://martinfowler.com/eaaDev/EventSourcing.html> (fetched 2026-08-21).

Established facts (quoted/paraphrased):
- "Event Sourcing ensures that all changes to application state are
  stored as a sequence of events… we can use the event log to
  reconstruct past states."
- "Since an application state is purely derivable from the event log,
  you can cache it anywhere you like." — snapshots are rebuildable.
- Version control systems are cited as the canonical example of this
  pattern; audit logging is a first-class motivation ("it's easy to
  serialize the events to make an Audit Log").

Application: the task system is structurally event-sourced already —
task completions are append-only events (one evidence file + one
`completed[]` entry each), and TASK_STATE.json is derived state that
could be rebuilt by replaying evidence. The current gap: completion
events live only inside a *rewritten* TASK_STATE.json, not in an
append-only event log (see §5 decisions).

### 3.3 Atomic file replacement (state-pointer durability)

Source: POSIX `rename(2)` — IEEE Std 1003.1-2017,
<https://pubs.opengroup.org/onlinepubs/9699919799/functions/rename.html>
(fetched earlier this session for the audit-ring retention work):
rename "specifies behavior when the *new* argument names an existing
file… the action of the function be atomic"; a link named `new`
"shall remain visible… throughout the renaming operation".

Established fact: write-temp-then-rename is the standard crash-safe
file update on POSIX filesystems (same technique used by this repo's
audit-retention archives: `retention.py` writes `.tmp` then
`os.replace`).

Application: `complete_task.py` currently rewrites TASK_STATE.json in
place (`json.dump` to the open file). A crash mid-write corrupts the
pointer. The fix pattern is already proven in-tree (retention archives).

### 3.4 SQLite (in-repo precedent for durable structured state)

Fact from codebase: the audit ring and PEP grants already use SQLite in
WAL mode with `synchronous=FULL` (`code/aiosh-cli/src/audit.ts`,
`code/aiosh-mcp/aiosh_mcp/audit_client.py`), giving transactional,
crash-safe, single-file state with a proven cross-substrate contract.
Relevant if the pointer/ledger ever needs concurrent writers.

## 4. Constraints (binding)

- **C-1 No-skip law** (GOALS.md, user mandate): only `next_task` may
  start; completion must advance by exactly 1. Any data-model change
  must keep this mechanically enforceable.
- **C-2 Reproducibility** (REP): the ledger must be regenerable
  deterministically from `tools/generate_master_tasks.py`.
- **C-3 No fabrication** (REP / constitution): tasks beyond the
  detailed near-term slice are structured envelopes, not invented
  specifics; their instructions are the process contract, not claims.
- **C-4 Evidence-before-completion**: a task's acceptance criteria must
  have recorded evidence before `complete_task.py` may advance.
- **C-5 Cross-agent handoff**: a fresh agent must derive the exact next
  action from TASK_STATE.json alone (boot procedure Step 5).

## 5. Facts vs assumptions

Facts (verified by reading/running the tree):
1. Ledger = 10,000 well-formed JSONL records, ids 1..10000 contiguous.
2. Pointer enforcement works (NO-SKIP refusal proven live).
3. Ledger `status` field is written once at generation time and is
   **never updated** by `complete_task.py` — truth of completion lives
   only in TASK_STATE.json `completed[]`. The two can drift.
4. TASK_STATE.json rewrite is non-atomic (in-place `json.dump`).
5. No file locking: two concurrent `complete_task.py` runs could both
   pass the `task_id == next_task` check and double-advance.
6. Legacy TASK_DATABASE.json self-declares non-authoritative; GOALS.md
   and README now forbid using it to pick work.

Assumptions (explicit, not verified facts):
- A1: single-writer operation (one agent session at a time). True today
  by process, not by mechanism.
- A2: 10,000-line single file stays practical to load/grep. At ~0.5–1
  KB/line (~5–10 MB) this is fine for text tools; no benchmark yet.
- A3: evidence files under `docs/tasks/evidence/` are the durable event
  log. They are human-written; nothing mechanically validates that an
  evidence file matches its task's acceptance criteria.

## 6. Unknowns / decisions needed (before any implementation task)

- **D1 — atomic pointer writes:** switch `complete_task.py` to
  tmp-file + `os.replace` (POSIX rename). Low risk, proven pattern
  in-tree. Recommend: yes.
- **D2 — status drift:** either (a) make `complete_task.py` also update
  the ledger JSONL line status (rewrite-on-complete, atomic), or
  (b) declare TASK_STATE.json the single source of truth for status and
  treat the ledger `status` field as generation-time-only. Recommend:
  (b) — cheaper, keeps the ledger immutable.
- **D3 — concurrency lock:** add `fcntl.flock` around the
  read-check-write of TASK_STATE.json. Needed only if multi-agent
  sessions become possible. Recommend: defer until A1 is invalidated;
  document the single-writer assumption instead.
- **D4 — append-only completion event log:** add
  `docs/tasks/COMPLETIONS.jsonl` (one event per completion: id, ts,
  note, evidence path) and derive TASK_STATE.json from it on demand.
  Makes the pointer rebuildable (Fowler §3.2) and gives the audit trail
  a machine-readable form. Recommend: yes, small change, high value.
- **D5 — blocked-task handling:** strictly linear means one blocked
  task halts all 9,999 behind it. Decide the escape valve: a recorded
  `blocked` event + human approval to advance, or a per-task
  `skip_reason` audit row. Recommend: blocked events stay recorded,
  advancement still requires explicit human override (never silent).
- **D6 — legacy DB disposition:** keep TASK_DATABASE.json frozen with
  its `authoritative: false` marker, or move it under `docs/archive/`.
  Recommend: freeze in place (repository evidence rule already points
  around it).

## 7. Source list

- JSON Lines — <https://jsonlines.org/> (fetched 2026-08-21)
- RFC 8259, The JavaScript Object Notation (JSON) Data Interchange
  Format — <https://www.rfc-editor.org/rfc/rfc8259> (§8.1 UTF-8/BOM)
- Fowler, M. "Event Sourcing" (2005) —
  <https://martinfowler.com/eaaDev/EventSourcing.html> (fetched 2026-08-21)
- POSIX rename(2), IEEE Std 1003.1-2017 —
  <https://pubs.opengroup.org/onlinepubs/9699919799/functions/rename.html>
- In-repo: `tools/generate_master_tasks.py`, `tools/complete_task.py`,
  `docs/tasks/*`, `mostimportanAIfolder/TASK_DATABASE.json` (metadata
  block), `code/aiosh-mcp/aiosh_mcp/retention.py` (tmp+rename precedent)
