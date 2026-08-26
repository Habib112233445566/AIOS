# SPEC — Task Ledger Control: Data Model & Operator Guide

**Status:** IMPLEMENTED (data model T-00011..T-00018; this document T-00019)
**Ship substrate:** Rust — `code/aiosh-rust/aiosh-core/src/ledger.rs`, exposed as
the production CLI surface `aiosh task …` (`code/aiosh-rust/aiosh-cli/src/main.rs`)
**Legacy reference:** `tools/task_ledger.py` (kept as the cross-substrate
contract oracle; behaviorally identical — parity asserted in CI)
**Full schema authority:** `docs/tasks/evidence/T-00012-data-model-specification.md`
**Research origin:** `docs/tasks/evidence/T-00011-data-model-research.md`
**Security review:** `docs/tasks/evidence/T-00017-security.md`
**Hardening:** `docs/tasks/evidence/T-00018-hardening.md`

Purpose: everything an operator or agent needs to *use* the ledger
correctly — what each file means, how to invoke the control commands,
what is enforced mechanically, and what the known limitations are.

---

## 1. What shipped (T-00014..T-00018)

- **Data model in Rust** (`aiosh-core/src/ledger.rs`) — a byte- and
  behavior-compatible port of `tools/task_ledger.py`: atomic state
  pointer updates, append-only fsync'd completion events, strict no-skip
  enforcement, block/unblock/skip, event-log rebuild, invariant check.
- **Production CLI surface** — `aiosh task status|done|block|unblock|
  skip|rebuild|check` (T-00016). Every invocation — including refusals
  and errors — writes one honest row to the SQLite audit ring
  (ADR-0035 §F-2 fail-open rule, T-00018).
- **Hardening** (T-00018): input size caps (ledger 64 MiB, events
  16 MiB, state 4 MiB); stale `TASK_STATE.json.tmp.<pid>` cleanup;
  error paths leave no temp-file leaks; `aiosh task` wired into the
  audit ring.
- **Cross-substrate parity** proven in BOTH directions (Rust writes →
  Python reads; Python writes → Rust reads) and asserted on every CI
  run inside `code/aiosh-rust/ci/rust_smoke.sh`.

## 2. Files (the data model)

All paths are relative to `docs/tasks/`:

| File | Mutability | Role |
|---|---|---|
| `MASTER_TASK_LEDGER.jsonl` | Immutable after generation (regenerate wholesale only) | Plan of record: 10,000 tasks, each `{id, title, phase, goal, instructions[], acceptance[], depends_on[], next_task}` |
| `COMPLETIONS.jsonl` | **Append-only**, fsync'd per line | Event log. One JSON object per mutation. Authoritative history |
| `TASK_STATE.json` | Rewritten atomically (`tmp.<pid>` + `rename`) | **Derived** live pointer: `next_task`, `completed[]`, `blocked[]`, `skipped[]`. Single source of truth for scheduling |
| `.TASK_STATE.lock` | Advisory `flock` target | Same-host single-writer insurance |
| `evidence/T-NNNNN*.md` | Append/new only | Per-task acceptance proof |

**Authoritative-status rule:** only `TASK_STATE.json` decides what may
start. The per-task `status` field inside the ledger JSONL is
generation-time metadata and MUST NOT be read for scheduling.
(`mostimportanAIfolder/TASK_DATABASE.json` is likewise non-authoritative.)

### 2.1 Event kinds (`COMPLETIONS.jsonl`)

`event ∈ {completed, blocked, unblocked, pointer_reset}`; `seq` starts
at 1 and increases by exactly 1 per line; `ts` is UTC ISO-8601.

```json
{"seq":18,"ts":"2026-08-21T11:46:19Z","event":"completed","task_id":18,
 "note":"…","evidence":["docs/tasks/evidence/T-00018-hardening.md"]}
```

### 2.2 State schema (`TASK_STATE.json`, schema_version 2)

```json
{
  "schema_version": 2,
  "ledger": "MASTER_TASK_LEDGER.jsonl",
  "total_tasks": 10000,
  "next_task": 19,
  "completed": [1, "..."],
  "blocked": [],
  "skipped": [],
  "last_completed_at": "2026-08-21T11:46:19Z",
  "last_event_seq": 18,
  "rule": "Execute ONLY next_task. ..."
}
```

A v1 file (no `schema_version`/`blocked`/`skipped`/`last_event_seq`) is
migrated transparently on read; the next write persists it as v2.

## 3. How to invoke (copy-pasteable)

The Rust binary is the shipping surface. Build once, then:

```bash
cargo build                                    # repo root of code/aiosh-rust
BIN=code/aiosh-rust/target/debug/aiosh
```

> **Path note:** unless the env var below is set, `aiosh task` resolves
> its data directory from the executable's own location, which lands at
> `<repo>/code/docs/tasks` for the standard `target/debug` layout (see
> §7 L2). Always set the override when driving the real ledger:

```bash
export AIOSH_TASKS_DIR="$PWD/docs/tasks"
```

Read operations:

```bash
$BIN task status        # print TASK_STATE.json (ok/data envelope)
$BIN task check         # validate ledger invariants (§4)
```

Complete the current task (the only sanctioned way to advance):

```bash
$BIN task done "$($BIN task status | python3 -c 'import json,sys;print(json.load(sys.stdin)["data"]["next_task"])')" \
     --note "one-line summary of what was verified" \
     --evidence docs/tasks/evidence/T-00019-docs.md
```

Block / resume / human-override (all require a non-empty reason):

```bash
$BIN task block   <id> --reason "waiting on X"   # id must BE next_task; pointer stays
$BIN task unblock <id> --reason "X resolved"     # removes from blocked[]; retry
$BIN task skip    <id> --reason "out of scope"   # pointer_reset event; advances past id
```

Disaster recovery — recompute the pointer from the append-only log:

```bash
$BIN task rebuild       # TASK_STATE.json is derived; events win
```

Legacy equivalent (identical contract, used by tests):

```bash
python3 tools/complete_task.py 19            # thin wrapper: complete + advance
python3 tools/task_ledger.py status|skip|... # full legacy surface
```

### 3.1 Verified refusal example (real output)

Out-of-order completion is mechanically refused with zero state change
(captured on a scratch copy, 2026-08-22):

```json
{
  "error": "NO-SKIP violation: attempted to complete T-00025 but next_task is T-00019. Complete T-00019 first.",
  "ok": false,
  "subcommand": "task done"
}
```

Blocking a non-current id is likewise refused: `"can only block
next_task (T-00019), got T-00020"`. Both refusals still write their
audit-ring row.

## 4. Enforced ledger invariants (`task check`)

For every consecutive pair of ledger lines (spec §2.1, pinned at
generation time):

- ids are exactly `1..N`, contiguous, unique;
- `depends_on == [id-1]` for `id > 1` (linear chain), `[]` for id 1;
- embedded `next_task == id + 1` where present.

`task check` returns `{"ok": true, "total_tasks": N}` or the first
violating line with the reason.

## 5. Ordering & crash guarantees

1. Acquire exclusive `flock` on `.TASK_STATE.lock`.
2. Read state; refuse if `id != next_task` (no-skip, zero mutation).
3. Stream-scan the ledger for the id (unknown id ⇒ refuse).
4. **Event first**: append the JSONL line, flush + `fsync`.
5. **State second**: write `TASK_STATE.json.tmp.<pid>` (cleaning stale
   tmp files first), `fsync`, atomic `rename(2)`.
6. Ensure the evidence stub `T-<id:05d>-completion.md` exists (never
   overwrites an existing evidence file).

Crash between 4 and 5 ⇒ event logged, pointer stale ⇒ `task rebuild`
repairs (the event log is authoritative). Crash between 5 and 6 ⇒
cosmetic only. Events are never rewritten or deleted (Constitution P-2
philosophy applied to the task ledger).

## 6. Security posture (summary; details in T-00017/T-00018)

- Path traversal / injection vectors reviewed and closed empirically
  (T-00017); ledger and events are parsed strictly, corrupt lines abort
  with the line number.
- Size caps bound memory on hostile/pathological files (T-00018):
  ledger 64 MiB · events 16 MiB · state 4 MiB.
- Error paths clean up their own temp files; stale temps from crashed
  writers are removed on the next save (never touches the live state).
- Every `aiosh task` outcome — success, refusal, error — produces an
  honest SQLite audit-ring row (ADR-0035 §F-2).

## 7. Known constraints and limitations

- **L1 — Single-writer assumption (D3).** `flock` serializes same-host
  writers; since T-00028 the wait is **bounded at 5 s** (`LOCK_NB`
  poll), after which the loser fails loudly with
  `ledger lock busy after 5000ms …` (standard envelope + honest audit
  row) instead of hanging forever. True multi-agent concurrency still
  needs a lock service; out of scope (spec §8).
- **L2 — RESOLVED (T-00024, D3 fix).** Path resolution now walks
  ancestors of `current_exe()` looking for
  `docs/tasks/MASTER_TASK_LEDGER.jsonl` and fails loudly with
  `cannot locate docs/tasks (set AIOSH_TASKS_DIR)` if unresolvable —
  no more wrong-directory guesses. `AIOSH_TASKS_DIR` remains the
  highest-priority override. Verified live from an arbitrary cwd.
- **L3 — RESOLVED (T-00024, D4 replay).** `rebuild` replays events in
  order reproducing LIVE pointer transitions exactly
  (completed⇒t+1, unblocked⇒t, pointer_reset⇒t+1, clamp past total).
  Skips now survive rebuilds in **both** substrates; parity proven
  both directions in CI (rust_smoke).
- **L4 — Evidence is attested, not validated.** The ledger records
  evidence *paths*; no machine check verifies content against the
  acceptance criteria (spec §8). Partially mitigated since
  T-00101..T-00110: `task validate` now machine-checks that referenced
  evidence paths EXIST (and flags orphans) — see §9 — but content-vs-
  acceptance validation remains out of scope.

## 9. Recovery & validation — `task validate` (T-00101..T-00110)

Read-only integrity report comparing the live `TASK_STATE.json` against a
deterministic replay of the append-only event log. **Report-only by
design**: it never mutates state, events, or evidence; `task rebuild`
(§3, lock-free by M5 design fact) remains the ONLY repair path.
Contract: `docs/tasks/evidence/T-00102-spec.md`.

Checks (stable additive key set, byte-parity across Rust MCP / Python MCP /
both CLIs):

| check | severity | detects |
|---|---|---|
| `state_vs_events` | fatal | hand-edited pointer, crash-window drift, partial writes |
| `event_seq` | fatal | non-contiguous seq, `last_event_seq != len(events)` |
| `pointer_range` | fatal | replayed pointer on completed/blocked id; beyond total |
| `evidence` | warning-only | referenced-but-missing paths, orphan completion stubs |

Hardening (T-00108): evidence paths are satisfiable ONLY if relative and
confined to the tasks dir / repo root — absolute or `..`-escaping strings
are always reported missing (existence-oracle closed; contents never read).

Copy-pasteable invocation:

```bash
# CLI (Rust ship path):
aiosh task validate

# Python reference:
python3 tools/task_ledger.py validate

# MCP wire (agent surface, grant-free):
# {"name":"aios.task","arguments":{"action":"validate"}}
```

Example output shape (trimmed):

```json
{"ok": true, "action": "validate", "consistent": true,
 "checks": {"state_vs_events": {"status": "ok", "detail": null, "fields": []},
            "event_seq": {"status": "ok", "detail": null},
            "pointer_range": {"status": "ok", "detail": null},
            "evidence": {"status": "warning",
                         "missing": ["T-00001:docs/tasks/evidence/T-00001"],
                         "orphans": []}},
 "replay": {"next_task": 104}, "live": {"next_task": 104}}
```

Known limitations: single corrupt event line still fails loudly via the
shared reader (no partial findings invented); evidence existence is
checked, not content (L4); findings are advisory — operators decide
between `rebuild` and manual repair. Evidence chain:
research `T-00101-research.md`, spec `T-00102-spec.md`, scaffold
`T-00103-scaffold.md`, implementation `T-00104-implementation.md`, unit
tests `T-00105-unit-test.md`, integration `T-00106-integration.md`,
security `T-00107-security-review.md`, hardening
`T-00108-hardening.md` (all under `docs/tasks/evidence/`).
- **L5 — RESOLVED (T-00041..T-00050).** Both substrates now expose the
  ledger over MCP (`aios.task` on Rust; `aios_task` on the Python
  reference server, §8.2) with one grant valid across both. rust_smoke
  additionally runs FOUR file-level cross-substrate flows (done/block/
  unblock+skip/rebuild replay); the P-suite pins tool-level behavior on
  the Python side and C-suite on the CLI.

## 8. Core service — `aios.task` MCP surface (T-00021..T-00030)

The agent loop can now manage this same ledger through the standard
gate (classifier → PEP → audit), per spec
`docs/tasks/evidence/T-00022-spec.md`:

| action | grant? | effect |
|---|---|---|
| `status`, `check`, `metrics`, `validate` | no | read state / validate invariants / consolidated snapshot / integrity report |
| `done` | **yes** | complete current task (+`note`, optional `evidence[]`) |
| `block`, `unblock`, `skip` | **yes** | hold / retry / human-override (+`reason`) |
| `rebuild` | **yes** | recompute pointer from event log (D4 replay) |

Schema violations (unknown keys, bad enum, empty/oversized strings,
`task_id < 1`, >16 evidence items) are protocol errors `-32602`;
request lines over 1 MiB are `-32700`; semantic refusals (NO-SKIP,
missing note/reason) come back as normal results with `isError: true`
so models can self-correct. Every call — ok, refused, or error — writes
exactly one audit-ring row; concurrent writers resolve within 5 s or
fail loudly with `ledger lock busy`.

Copy-pasteable (from repo root, verified 2026-08-22):

```bash
# 1. Read the live pointer over MCP:
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"ops","version":"1"}}}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"aios.task","arguments":{"action":"status"}}}' \
 | code/aiosh-rust/target/debug/aiosh-mcp

# → result.structuredContent.result = {"ok":true,"action":"status",
#    "data":{"next_task":29,…},"audit_id":97,…}

# 2. Mint a grant so an AGENT may advance the queue (human consent, D7):
code/aiosh-rust/target/debug/aiosh grant create --to agent --tools "aios.task" --ttl 3600

# 3. Agent-side mutation (grant_id from step 2):
#    {"name":"aios.task","arguments":{"action":"done","task_id":29,
#     "note":"…","evidence":["docs/tasks/evidence/x.md"],"grant_id":"gr_…"}}
```

Security posture: see `docs/tasks/evidence/T-00027-security.md`
(grant-scope isolation, hostile-payload inertness, u64 extremes,
chain-integrity-after-abuse — all empirically verified).

### 8.1 CLI surface — unified validation (T-00031..T-00040)

`aiosh task` now runs the SAME validation as the MCP tool (one source:
`task_service::TaskCall`). Behavior changes vs the pre-T-00034 CLI,
all intentional (spec `docs/tasks/evidence/T-00032-spec.md`, D1–D5):

- `done --note ""` / missing `--note` → **refused** (was silently
  stored an empty note).
- Text values ≤4096 bytes; evidence ≤16 items, each ≤4096.
- A value starting with `--` is refused unless preceded by a lone `--`
  (`--reason -- --weird-reason` stores the literal); unknown options
  are named in the error. POSIX-G9 deviation (options after operands)
  kept and documented.

Copy-pasteable (live-verified 2026-08-22):

```bash
export AIOSH_TASKS_DIR="$PWD/docs/tasks"   # optional; resolver auto-detects
code/aiosh-rust/target/debug/aiosh task help          # full usage, exit 0
code/aiosh-rust/target/debug/aiosh task status        # {"ok":true,"data":{…}}
code/aiosh-rust/target/debug/aiosh task done "$(...)" --note "what was verified"
# refusal example (stderr): {"error":"'note' must be non-empty when present","ok":false}
```

Every outcome — including usage refusals — writes one honest audit row;
non-UTF-8 argv is lossy-converted, never a panic (T-00038). Stream
convention: success → stdout, refusal → stderr.

### 8.2 Python reference server — cross-substrate parity (T-00041..T-00050)

The legacy Python MCP server (`code/aiosh-mcp`) now registers the same
surface as **`aios_task`** (underscore naming per substrate convention,
research F5/A1): identical 7 actions, identical grant policy
(`status/check` free; mutations need a grant covering `"aios.task"`),
identical caps (4096/16), identical envelopes, and the SAME gate string
— so one grant authorizes both servers. Validation runs pre-gate;
every call commits exactly one honest row via `_dispatch`; ledger ops
delegate to `tools/task_ledger.py` (bounded lock + D4 replay shared).
Hardened: module cached per process, loader failures audited, bool
`task_id` rejected.

Cross-substrate proof (live 2026-08-22): a grant minted with the RUST
CLI was spent on the PYTHON tool (T-00046/P-suite), and the P8 leg
replayed a skip through rebuild on the Python path. L5's residual is
now closed: both substrates expose the ledger to agents.

Copy-pasteable (from `code/aiosh-mcp`, read-only, live-verified):

```bash
python3 - <<'PY'
from aiosh_mcp.server import aios_task
print(aios_task(action="check"))
# {"ok": true, "total_tasks": 10000, "action": "check", "audit_id": 215}
PY
```

### 8.3 Configuration — `AIOSH_LEDGER_*` environment (T-00051..T-00060)

Six operational knobs are configurable via environment variables
(Twelve-Factor III: config in env, not files — 12factor.net/config,
fetched 2026-08-22). Defaults equal the shipped constants; precedence
env > default; read fresh at each operation; invalid/out-of-range
values fail LOUDLY naming the variable (audited on CLI). Ceiling:
lock timeout ≤ 86400 s.

| Variable | Default | Floor |
|---|---|---|
| `AIOSH_LEDGER_LOCK_TIMEOUT_SECS` | 5 | 1 (ceiling 86400) |
| `AIOSH_LEDGER_MAX_LEDGER_BYTES` | 67108864 | 1024 |
| `AIOSH_LEDGER_MAX_EVENTS_BYTES` | 16777216 | 1024 |
| `AIOSH_LEDGER_MAX_STATE_BYTES` | 4194304 | 1024 |
| `AIOSH_LEDGER_MAX_TEXT` | 4096 | 64 |
| `AIOSH_LEDGER_MAX_EVIDENCE_ITEMS` | 16 | 1 |

Both substrates read these identically. Deliberately NO MCP tool
exposes config — knobs are operator-only (agent-mutable security
knobs would be anti-security).

Copy-pasteable (live-verified 2026-08-22):

```bash
export AIOSH_LEDGER_LOCK_TIMEOUT_SECS=15     # slower storage? widen the wait
export AIOSH_LEDGER_MAX_TEXT=8192            # allow longer completion notes
code/aiosh-rust/target/debug/aiosh task config
# {"ok":true,"subcommand":"task config","data":{
#   "max_text":{"source":"env","value":8192},
#   "lock_timeout_secs":{"source":"default","value":5}}, …}
```

Constraints/limitations: unknown `AIOSH_LEDGER_*` vars are ignored;
the published MCP inputSchema keeps the default bounds (env can only
effectively tighten wire clients); extreme lock timeouts above the
ceiling are refused rather than clamped.

### 8.4 Automated tests — the ledger regression matrix (T-00061..T-00070)

Five suites pin this component (all wired into
`ci/run_all_smokes.sh`, 15 total):

| Suite | Surface | Cases |
|---|---|---|
| `tools/test_task_ledger.py` | legacy module + CLI | U1..U16 |
| `code/aiosh-mcp/tests/test_task_service_smoke.py` | Rust MCP tool | W1..W8 |
| `code/aiosh-cli/tests/test_task_cli_smoke.py` | CLI | C1..C9 |
| `code/aiosh-mcp/tests/test_task_mcp_smoke.py` | Python MCP tool | P1..P8 |
| `code/aiosh-cli/tests/test_task_config_smoke.py` | env-config | K1..K5 |
| `code/aiosh-mcp/tests/test_ledger_matrix_smoke.py` | **cross-surface** | M1..M8 |

The matrix suite (T-00063..T-00066) pins what single-surface tests
cannot: one grant valid on BOTH servers; narrow-grant rejection;
concurrent-writer lock-busy with bounded wait; config propagation into
the Python surface; grant expiry fail-closed on both substrates; and
block/unblock pointer semantics. Conventions for new cases: repo
PASS/FAIL style, isolated temp sandboxes, observable-behavior
assertions, a broken-feature check before landing, explicit subprocess
timeouts.

Known fact encoded by M5: **`rebuild` is lock-free by design**
(recovery tool, spec T-00012 §4) — do not "fix" its lack of locking.

### 8.5 Security policy (T-00071..T-00080)

Root [`SECURITY.md`](../SECURITY.md) defines: vulnerability scope
(gate bypass, chain break, sandbox escape, no-skip violation, gate-
flipping prompt injection, secret exposure), the private reporting
channel (GitHub Security Advisory, owner-provided), supported
surfaces, and a 7-day ack / 90-day coordinated-disclosure commitment.
Enforced by the `security_policy` CI suite
(`tools/check_security_policy.py`): URL removal, marker-word
reintroduction, or broken links fail the baseline. Rule-pack changes require a
`CLASSIFIER_REVISION` bump. Index of all seven component security
reviews lives in the policy itself.

### 8.6 Observability — the `metrics` action (T-00081..T-00090)

`aios.task {action:"metrics"}` (CLI: `aiosh task metrics`) returns ONE
consolidated snapshot with the STABLE additive-only key set
`data = {tasks, audit, config}`:

- `tasks` — counters only: `total_tasks`, `completed`/`blocked`/
  `skipped` (ints), `next_task`, `last_event_seq`, `last_completed_at`.
  **No titles, ids, notes, or evidence paths are exposed.**
- `audit` — live ring facts: `rows` (COUNT(*), O(1)),
  `verify_ok` (light live-chain verify), `head_hash_prefix` (12 hex).
- `config` — the six effective `AIOSH_LEDGER_*` values (operator's own
  knobs; agents cannot change them over MCP, SPEC §8.3).

Grant policy: read-only, NO grant required (D1 truth table). Every
call — ok, refused, or error — commits exactly one audit row on the
Rust surfaces; the Python reference refuses invalid shapes pre-gate
(no row), matching its documented §8.2 convention. `metrics` takes NO
`task_id` (refused on all surfaces, T-00085 O4–O6).

Copy-pasteable (live-verified 2026-08-22):

```bash
# CLI:
code/aiosh-rust/target/debug/aiosh task metrics
# → {"ok":true,"subcommand":"task metrics","data":{"tasks":{…},
#    "audit":{"rows":97,"verify_ok":true,"head_hash_prefix":"d2f1…"},
#    "config":{…}},"audit_id":…}

# MCP wire (after the initialize handshake):
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":
 {"name":"aios.task","arguments":{"action":"metrics"}}}
```

Limitations (honest):
- L-O1: `verify_ok` runs a live-chain walk per call — O(live rows);
  bounded by retention `keep_rows` (ADR-0036), not by total history.
- L-O2: Python pre-gate validation failures carry no `audit_id`; Rust
  refusals always do. Envelope parity is asserted by
  `tests/test_metrics_smoke.py` (O-suite).
- L-O3: the snapshot is eventually-consistent with concurrent writers
  (single read pass; no lock is taken for the read-only action).

Evidence: spec `docs/tasks/evidence/T-00082-spec.md` · scaffold
T-00083 · implementation T-00084 · unit tests
`tests/test_metrics_smoke.py` + `T-00085-observability-unit-test.md` ·
integration `T-00086-integration.md` · security `T-00087-security.md` ·
hardening `T-00088-hardening.md` · epic verification T-00090.

## 9. Evidence index (Task Ledger Control epic)

| Task | Artifact |
|---|---|
| T-00011 research | `docs/tasks/evidence/T-00011-data-model-research.md` |
| T-00012 specification | `docs/tasks/evidence/T-00012-data-model-specification.md` |
| T-00013 CI runner | `ci/run_all_smokes.sh` |
| T-00014 implementation | `code/aiosh-rust/aiosh-core/src/ledger.rs`, `docs/tasks/evidence/T-00014-data-model-implementation.md` |
| T-00015 integration | `code/aiosh-rust/aiosh-cli/src/main.rs` (`cmd_task`), `docs/tasks/evidence/T-00015.md` |
| T-00016 wiring | `docs/tasks/evidence/T-00016.md` (`aiosh task status/done/block/unblock/skip/rebuild/check`) |
| T-00017 security | `docs/tasks/evidence/T-00017-security.md` |
| T-00018 hardening | `docs/tasks/evidence/T-00018-hardening.md` |
| T-00019 this document (data model) | `docs/SPEC-TASK-LEDGER.md`, `docs/tasks/evidence/T-00019-docs.md` |
| T-00020 epic verification | `docs/tasks/evidence/T-00020-verify.md` |
| T-00021 core-service research | `docs/tasks/evidence/T-00021-research.md` |
| T-00022 core-service spec | `docs/tasks/evidence/T-00022-spec.md` |
| T-00023 core-service scaffold | `docs/tasks/evidence/T-00023-scaffold.md` |
| T-00024 core-service implementation | `docs/tasks/evidence/T-00024-implementation.md` |
| T-00025 core-service unit tests | `docs/tasks/evidence/T-00025-unit-tests.md` |
| T-00026 core-service integration | `docs/tasks/evidence/T-00026-integration.md` |
| T-00027 core-service security review | `docs/tasks/evidence/T-00027-security.md` |
| T-00028 core-service hardening | `docs/tasks/evidence/T-00028-hardening.md` |
| T-00029 core-service documentation | `docs/SPEC-TASK-LEDGER.md` §8, `docs/tasks/evidence/T-00029-docs.md` |
| T-00030 core-service verification | `docs/tasks/evidence/T-00030-verify.md` |
| T-00031 CLI-surface research | `docs/tasks/evidence/T-00031-research.md` |
| T-00032 CLI-surface spec | `docs/tasks/evidence/T-00032-spec.md` |
| T-00033 CLI-surface scaffold | `docs/tasks/evidence/T-00033-scaffold.md` |
| T-00034 CLI-surface implementation | `docs/tasks/evidence/T-00034-implementation.md` |
| T-00035 CLI-surface unit tests | `docs/tasks/evidence/T-00035-unit-tests.md` |
| T-00036 CLI-surface integration | `docs/tasks/evidence/T-00036-integration.md` |
| T-00037 CLI-surface security review | `docs/tasks/evidence/T-00037-security.md` |
| T-00038 CLI-surface hardening | `docs/tasks/evidence/T-00038-hardening.md` |
| T-00039 CLI-surface documentation | `docs/SPEC-TASK-LEDGER.md` §8.1, `docs/tasks/evidence/T-00039-docs.md` |
| T-00040 CLI-surface verification | `docs/tasks/evidence/T-00040-verify.md` |
| T-00041 MCP/API research | `docs/tasks/evidence/T-00041-research.md` |
| T-00042 MCP/API spec | `docs/tasks/evidence/T-00042-spec.md` |
| T-00043 MCP/API scaffold | `docs/tasks/evidence/T-00043-scaffold.md` |
| T-00044 MCP/API implementation | `docs/tasks/evidence/T-00044-implementation.md` |
| T-00045 MCP/API unit tests | `docs/tasks/evidence/T-00045-unit-tests.md` |
| T-00046 MCP/API integration | `docs/tasks/evidence/T-00046-integration.md` |
| T-00047 MCP/API security review | `docs/tasks/evidence/T-00047-security.md` |
| T-00048 MCP/API hardening | `docs/tasks/evidence/T-00048-hardening.md` |
| T-00049 MCP/API documentation | `docs/SPEC-TASK-LEDGER.md` §8.2, `docs/tasks/evidence/T-00049-docs.md` |
| T-00050 MCP/API verification | `docs/tasks/evidence/T-00050-verify.md` |
| T-00051 configuration research | `docs/tasks/evidence/T-00051-research.md` |
| T-00052 configuration spec | `docs/tasks/evidence/T-00052-spec.md` |
| T-00053 configuration scaffold | `docs/tasks/evidence/T-00053-scaffold.md` |
| T-00054 configuration implementation | `docs/tasks/evidence/T-00054-implementation.md` |
| T-00055 configuration unit tests | `docs/tasks/evidence/T-00055-unit-tests.md` |
| T-00056 configuration integration | `docs/tasks/evidence/T-00056-integration.md` |
| T-00057 configuration security review | `docs/tasks/evidence/T-00057-security.md` |
| T-00058 configuration hardening | `docs/tasks/evidence/T-00058-hardening.md` |
| T-00059 configuration documentation | `docs/SPEC-TASK-LEDGER.md` §8.3, `docs/tasks/evidence/T-00059-docs.md` |
| T-00060 configuration verification | `docs/tasks/evidence/T-00060-verify.md` |
| T-00061 automated-tests research | `docs/tasks/evidence/T-00061-research.md` |
| T-00062 automated-tests spec | `docs/tasks/evidence/T-00062-spec.md` |
| T-00063 automated-tests scaffold | `docs/tasks/evidence/T-00063-scaffold.md` |
| T-00064 automated-tests implementation | `docs/tasks/evidence/T-00064-implementation.md` |
| T-00065 automated-tests unit tests | `docs/tasks/evidence/T-00065-unit-tests.md` |
| T-00066 automated-tests integration | `docs/tasks/evidence/T-00066-integration.md` |
| T-00067 automated-tests security review | `docs/tasks/evidence/T-00067-security.md` |
| T-00068 automated-tests hardening | `docs/tasks/evidence/T-00068-hardening.md` |
| T-00069 automated-tests documentation | `docs/SPEC-TASK-LEDGER.md` §8.4, `docs/tasks/evidence/T-00069-docs.md` |
| T-00070 automated-tests verification | `docs/tasks/evidence/T-00070-verify.md` |
| T-00071 security-policy research | `docs/tasks/evidence/T-00071-research.md` |
| T-00072 security-policy spec | `docs/tasks/evidence/T-00072-spec.md` |
| T-00073 security-policy scaffold | `SECURITY.md`, `docs/tasks/evidence/T-00073-scaffold.md` |
| T-00074 security-policy implementation | `SECURITY.md`, `docs/tasks/evidence/T-00074-implementation.md` |
| T-00075 security-policy unit tests | `tools/check_security_policy.py`, `docs/tasks/evidence/T-00075-unit-tests.md` |
| T-00076 security-policy integration | `ci/run_all_smokes.sh`, README, `docs/tasks/evidence/T-00076-integration.md` |
| T-00077 security-policy review | `docs/tasks/evidence/T-00077-security.md` |
| T-00078 security-policy hardening | `docs/tasks/evidence/T-00078-hardening.md` |
| T-00079 security-policy documentation | `docs/SPEC-TASK-LEDGER.md` §8.5, `docs/tasks/evidence/T-00079-docs.md` |
