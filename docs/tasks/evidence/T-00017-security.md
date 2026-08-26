# T-00017 — Task Ledger Control data model: Security Review

**Completed:** 2026-08-21
**Subject:** `code/aiosh-rust/aiosh-core/src/ledger.rs` + `aiosh task`
CLI surface (`code/aiosh-rust/aiosh-cli/src/main.rs`). The ledger is the
single-writer control plane for `docs/tasks/` (TASK_STATE.json,
COMPLETIONS.jsonl, MASTER_TASK_LEDGER.jsonl).

## 1. Input validation

| Input | Validation | Result |
|---|---|---|
| `task_id` | strict `u64` parse at the CLI boundary (`parse::<u64>()`); no fallback | `'../evil'`, `'12;rm -rf /'` → rejected with `done requires <task_id>` |
| oversized `task_id` (u64::MAX) | parsed, then no-skip guard rejects (pointer unchanged) | refusal, no state mutation |
| `--note` / `--reason` | opaque strings; **JSON-escaped** by `serde_json::to_string` into the event line | newline/injection payload stays one escaped line |
| ledger lines (untrusted file) | strict JSON parse; corrupt line → `ValueError`-style error, never skipped silently | `check`/`read_events` surface the exact line |

**Empirically verified:** `aiosh task done 17 --note 'line1\n{"seq":999,...}'`
produced exactly one event record with the note escaped as
`line1\n{"seq":999,...}` — no forged event line, seq counter intact
(17 events, seq 1..17).

## 2. Path / argument injection

- **Path traversal:** evidence-stub filenames are derived ONLY from the
  validated numeric task id (`format!("T-{:05}-completion.md", id)`) under
  the fixed `evidence/` dir — user strings never reach a filesystem path.
  Verified: no `evil`/`..`/`999` artifacts in `evidence/`.
- **Temp-file safety (`save_state_atomic`):** `<path>.tmp.<pid>` opened
  with `create_new` (O_EXCL) and mode 0o644 — no symlink-follow on the
  temp, then `rename` for atomic replacement (POSIX rename semantics).
- **No shell anywhere:** the ledger never builds or executes a command
  string; no argument can reach a shell.

## 3. Untrusted-content handling

- Event log and state are parsed with strict JSON; `read_events` reports
  `corrupt event log line N` rather than skipping.
- `--note`/`--reason` are stored, never interpreted (no markdown/HTML
  rendering in the data model; the evidence stub writes them verbatim
  as plain text).

## 4. State-changing paths: locking + audit emission

**Audit-row model (explicit scope decision):** the task ledger's
canonical audit record is its **own append-only event log
(`COMPLETIONS.jsonl`)** — not the SQLite audit ring. This matches the
original `tools/task_ledger.py` design (the Python tool also never
writes to the audit ring) and the spec (T-00012 D-series). Every
state-changing path appends an event **before** the pointer moves
(event-then-state, single-writer under flock, D3); `last_event_seq` is
monotonic and a tampered state file can be reconstructed via `rebuild`.
The SQLite audit ring covers host-action tools (MCP/CLI surfaces); the
ledger is an internal control-plane data model whose state transitions
are fully auditable from its own event stream.

**PEP gating (explicit scope decision):** the classifier→PEP gate is
not applied to `aiosh task` — the ledger performs no host actions, no
process execution, and no external-target interaction, so there is no
action for PEP to authorize. The gate's purpose (authorize host actions)
does not apply to a data-model CLI; the mechanical no-skip law is the
ledger's access-control mechanism. If `aiosh task` is ever exposed as
an MCP tool, it MUST be routed through the standard gate (note for
T-00019 docs).

| Operation | Lock | Event written | Pointer effect |
|---|---|---|---|
| `done` | flock EX | `completed` | advance by exactly 1 |
| `block` | flock EX | `blocked` | unchanged |
| `unblock` | flock EX | `unblocked` | back to task |
| `skip` | flock EX | `pointer_reset` | advance by 1 |
| `rebuild` | — (read-only recompute) | — | recomputed from events |

## 5. Error handling

- All failures return the standard envelope
  `{"ok": false, "error": <reason>}` with non-zero exit (no silent
  failure, no partial writes — state is only replaced atomically on the
  success path).
- No resource leaks on the error path: `FileLock` is RAII (unlock on
  drop), temp state files are created with `create_new` and renamed or
  never left behind, DB/file handles close on scope exit.

## 6. Abuse scenarios assessed

| Scenario | Verdict |
|---|---|
| Attacker completes out-of-order tasks | **Blocked** — mechanical no-skip refusal, pointer unchanged |
| Forged event line via note/reason content | **Blocked** — JSON escaping keeps one record per line; seq is server-assigned |
| Path traversal via task_id / evidence path | **Blocked** — u64 parse + `T-{:05}` formatting only |
| Symlink race on the state temp file | **Blocked** — `create_new` (O_EXCL) refuses existing/symlink targets |
| Concurrent writers corrupt the pointer | **Blocked** — exclusive `flock` around every mutation |
| Tampered state file after the fact | **Detectable** — `rebuild` reconstructs from the event log; `check` validates ledger invariants |
| Local attacker with write access to `docs/tasks/` | **Out of scope** (documented) — the ledger trusts the local user; a full-DB-compromise attacker can rewrite anything, including the Python tool |
| `AIOSH_TASKS_DIR` env override | **Documented** — deliberately supported for sandboxed tests/CI; it is a local trust boundary, not a security boundary |

## Conclusion

No open policy bypass found. The no-skip law is enforced mechanically at
the only mutation boundary (the CLI), injection and traversal vectors are
closed, and every state change leaves an append-only audit trail that can
be independently rebuilt. Hardening follow-ups (timeouts, size caps,
cleanup guarantees) are tracked in T-00018.
