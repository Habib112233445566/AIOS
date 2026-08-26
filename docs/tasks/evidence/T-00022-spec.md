# T-00022 — Task Ledger Control core service: Specification

**Date:** 2026-08-22
**Type:** specification (no code changed)
**Depends on:** T-00021 research (`T-00021-research.md`, facts F1–F9,
external E1–E6, decisions D1–D7)
**Status:** SPECIFIED — all decisions below resolved with the project
owner's standing approval ("proceed" on proposed defaults, 2026-08-22)

> Every new API in this document is an **AIOS-specific proposal** built
> on AIOS's own `dispatch`/PEP/audit contracts. Upstream-facing behavior
> (tool naming, inputSchema, isError semantics) conforms to the MCP
> Tools spec as fetched 2026-08-22 (E1–E5).

---

## 1. Resolved decisions (from research §6)

| ID | Decision | Resolution |
|---|---|---|
| D1 | Grant policy | `status`, `check` read-only (`require_grant=false`). `done`, `block`, `unblock`, `skip`, `rebuild` are consequential (**`require_grant=true`**) |
| D2 | Surface shape | **One grouped tool** `aios.task` with an `action` argument (7 actions). Manifest grows 12 → 13 |
| D3 | L2 path-resolution fix | **In scope**, folded into Implementation (T-00024) as shared `tasks_dir()` repair |
| D4 | L3 rebuild-vs-skip fix | **In scope**: `rebuild` gains deterministic pointer replay in **both** substrates (Rust + Python reference), preserving cross-substrate parity |
| D5 | Protocol pin | Server stays pinned to `protocolVersion "2025-06-18"` echo behavior (unchanged code; E6 noted) |
| D6 | Rate limiting (MCP MUST) | Disposition documented (§8): single-writer `flock` serializes mutations; local stdio transport; host owns throttling. Revisit when any remote transport lands |
| D7 | Human-in-the-loop | The PEP grant is the consent artifact (existing `aios.audit.rotate` precedent); mandatory non-empty `note`/`reason` fields give a visible trail in `COMPLETIONS.jsonl`. No extra UI channel this epic |

## 2. Components

| Piece | New/Reused | Location |
|---|---|---|
| Service module `task_service` | **New** (wraps existing `ledger.rs` functions — no logic duplication) | `code/aiosh-rust/aiosh-core/src/task_service.rs` |
| MCP tool `aios.task` | **New** registration in `tool_manifest()` + `call_tool()` | `code/aiosh-rust/aiosh-mcp/src/main.rs` |
| Ledger data model | **Reused verbatim** | `aiosh-core/src/ledger.rs` |
| Gate (classifier→PEP→audit) | **Reused verbatim** | `aiosh-core/src/dispatch.rs` (`dispatch`, `commit`, `recorded_call`) |
| PEP store | **Reused verbatim** (glob tool matching via `pep.check`) | `aiosh-core/src/pep.rs` |
| CLI `aiosh task …` | **Unchanged** (already shipped, T-00016/T-00018) | `aiosh-cli/src/main.rs::cmd_task` |
| Python ledger reference | Modified **only** for D4 replay parity + its unit tests | `tools/task_ledger.py`, `tools/test_task_ledger.py` |
| CI wire-contract smoke | Updated count assertion 12 → 13 + new task-tool cases | `code/aiosh-rust/ci/rust_smoke.sh` |

## 3. Tool contract (`aios.task`)

```jsonc
// tools/list entry (name/description/inputSchema; E1/E2-conformant)
{
  "name": "aios.task",
  "description": "Task Ledger Control: query or advance the AIOS master task ledger. Read-only: status, check. Consequential (require grant): done, block, unblock, skip, rebuild.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "action":   { "type": "string",
                    "enum": ["status","check","done","block","unblock","skip","rebuild"] },
      "task_id":  { "type": "integer", "minimum": 1 },
      "note":     { "type": "string", "minLength": 1, "maxLength": 4096 },
      "reason":   { "type": "string", "minLength": 1, "maxLength": 4096 },
      "evidence": { "type": "array", "items": {"type":"string"}, "maxItems": 16 },
      "grant_id": { "type": "string" }
    },
    "required": ["action"],
    "additionalProperties": false
  }
}
```

### 3.1 Action semantics

All actions resolve the tasks directory through the repaired resolver
(§5), acquire the same flock the CLI uses (single-writer, L1), and go
through the standard gate ordering (§4).

| action | Required args | Library call (reused) | Persistence effects | Grant |
|---|---|---|---|---|
| `status` | — | `ledger::load_state` | none (read) | no |
| `check` | — | `ledger::assert_ledger_invariants` | none (read) | no |
| `done` | `task_id`, `note` (≥1 char, **AIOS-specific strictness**) | `ledger::complete_task` | append `completed` event (fsync) → atomic `TASK_STATE.json` rewrite → evidence stub if absent | **yes** |
| `block` | `task_id`, `reason` | `ledger::block_task` | append `blocked` event → state rewrite; pointer held | **yes** |
| `unblock` | `task_id`, `reason` | `ledger::unblock_task` | append `unblocked` event → state rewrite; pointer returns to id | **yes** |
| `skip` | `task_id`, `reason` | `ledger::skip_task` | append `pointer_reset` event → state rewrite; pointer advances past id | **yes** |
| `rebuild` | — | `ledger::rebuild_state` (with D4 replay, §6) | full atomic state rewrite recomputed from `COMPLETIONS.jsonl` (log untouched) | **yes** |

### 3.2 Result envelope

Happy path (normal tool result, `isError:false`):

```json
{ "ok": true, "action": "done", "data": {"completed":23,"next_task":24,"title":"…"},
  "audit_id": 42 }
```

Business refusal / runtime failure (**`isError:true`**, model-correctable
per E4 — same envelope the CLI prints today):

```json
{ "ok": false, "action": "done", "error": "NO-SKIP violation: attempted to complete T-00025 but next_task is T-00023. Complete T-00023 first." }
```

Protocol errors stay protocol errors (unchanged server behavior):
unknown tool ⇒ `-32601`; malformed envelope (missing/non-enum `action`,
wrong types) ⇒ `-32602`.

### 3.3 Error taxonomy (all `ok:false`, zero persistence unless stated)

| Condition | Source | State effect |
|---|---|---|
| unknown/malformed `action` | inputSchema validation | none (protocol error −32602) |
| missing `task_id` where required | service pre-validation | none |
| missing/empty `note`(done) or `reason`(block/unblock/skip) | service pre-validation (ledger already enforces reason) | none |
| out-of-order completion | `complete_task` NO-SKIP guard | none |
| `block/unblock/skip` on non-current id | ledger guards | none (except legal skip of current) |
| state/ledger/events oversized (64/64/16 MiB caps) | T-00018 `read_capped` | none |
| corrupt event-log line | `read_events` reports line number | none |
| flock contention / fs errors | OS error surfaced verbatim | none |
| gate refusals (classifier refused / grant missing-invalid) | `dispatch` writes the refusal audit row, then service returns `ok:false` with `gate` reason | none (one audit row IS written — that is the honest trail, F6) |

## 4. Gate flow (per call, exact)

1. `classify("aios.task", None, args)` — rule pack has no task rules;
   expected pass-through with baseline flags (C4 auditable flag true),
   `policy_revision` recorded on the row.
2. If action ∈ mutating set: `require_grant=true` ⇒ `pep.check(grant_id, "aios.task", None)`
   must succeed against a grant whose `tools` glob covers `aios.task`.
   Missing/invalid ⇒ refusal row + `ok:false`.
3. Execute the §3.1 library call inside `recorded_call(...)` so exactly
   **one** audit row extends the chain for every outcome — ok, refused,
   or error (F4/F6). Actor: `agent:mcp@aiosh-mcp` (existing convention).

## 5. Tasks-directory resolution (D3, fixes L2)

Shared resolver moves into `task_service::tasks_dir()` and is adopted by
the CLI (`cmd_task`) so both surfaces behave identically:

1. `AIOSH_TASKS_DIR` env override (unchanged semantics);
2. else walk ancestors of `current_exe()`'s directory looking for
   `docs/tasks/MASTER_TASK_LEDGER.jsonl`; first hit wins;
3. else return the error `"cannot locate docs/tasks (set AIOSH_TASKS_DIR)"`
   — loud failure, never a wrong-directory guess.

Unit tests cover: env override wins; synthetic exe-tree resolves; bare
tempdir errors.

## 6. Rebuild pointer replay (D4, fixes L3)

`rebuild_state` becomes a deterministic event-order replay that
reproduces live pointer transitions exactly:

```text
next = 1
for ev in events (in seq order):
    completed(t)     => completed += t; last_ts = ts; next = t + 1
    blocked(t)       => blocked += t            (pointer unchanged)
    unblocked(t)     => blocked -= t; next = t  (live retry semantics)
    pointer_reset(t) => skipped += t; next = t + 1
```

Implemented identically in `ledger.rs` and `tools/task_ledger.py`;
new tests: Rust `rebuild_replays_skip_and_unblock_pointers`, Python
U14/U15; `rust_smoke.sh` parity step extended with a skip-then-rebuild
scenario (both directions) — also closing the flow-coverage half of L5.
`docs/SPEC-TASK-LEDGER.md` §7 L3 is rewritten in the Documentation task
(T-00029) to describe replay semantics instead of the rewind limitation.

## 7. Audit effects (summary)

Exactly one `audit_ring` row per `aios.task` call regardless of outcome:
fields `tool="aios.task"`, `command="task.<action>"`, canonical `args`,
actor `agent:mcp@aiosh-mcp`, constitution rev, policy revision +
classifier evidence, `outcome ∈ {ok, refused, error}` with detail.
Ledger-side events (`COMPLETIONS.jsonl`) remain the separate,
append-only business trail they are today. The two trails are
complementary: audit ring = who called the gate and what the gate said;
COMPLETIONS.jsonl = what changed in the plan-of-record state.

## 8. Non-functional dispositions

- **Rate limiting (E5 MUST, D6):** satisfied administratively for this
  epic — local stdio transport, host process owns the model loop,
  mutations serialized by the exclusive flock. Explicit limiter deferred
  until a network transport exists (recorded here so the MUST has an
  owner).
- **Input sanitization (E5 MUST):** schema-typed args only
  (`additionalProperties:false`), length caps above, ids bounded ≥1,
  strings passed verbatim into typed ledger APIs that never interpolate
  into paths or SQL (events/state are written by serde, not shell).
- **Deterministic ordering (E3):** `aios.task` appended after the five
  pentest entries; manifest order otherwise untouched.

## 9. Out of scope (unchanged limitations)

Daemon/service process (research Candidate B — rejected), multi-agent
lock service (L1), machine validation of evidence content (L4), remote
transport concerns. `skipped[]` visibility and `unblocked` retry flows
are unchanged apart from replay fidelity.

## 10. Reviewability check

- Happy path: §3.1 + §3.2 envelope.
- Failure paths: §3.3 taxonomy incl. gate refusals.
- Persistence effects: §3.1 column + §7 audit effects.
- Reused vs new: §2 table.
- A reviewer can validate this contract without opening the
  implementation.
