# T-00024 — Task Ledger Control core service: Implementation

**Date:** 2026-08-22
**Type:** implementation (Rust + Python-reference parity)
**Depends on:** T-00023 scaffold
**Spec:** `docs/tasks/evidence/T-00022-spec.md` (decisions D1–D7)

## Failing-test-first record

Two tests were written/adjusted before their fixes landed, and both
caught real defects:

1. **Python U8 pinned the old rewind bug** — it expected
   `next_task == 4` after the last task (3 of 3) was completed; live
   behavior is `None`. Under D4 replay the rebuild now reproduces live
   transitions, so U8's expectation was corrected to `is None`
   (justified by U7's live observation in the same run).
2. **New `rebuild_clamps_pointer_at_end_of_ledger`** initially called
   `skip_task(10)` at pointer 1 and was correctly refused by the
   no-skip guard — test rewritten to advance honestly first.

## What shipped

### Rust (`code/aiosh-rust`)

- **`aiosh-core/src/task_service.rs`** — implemented per spec:
  - `TaskAction::parse/as_str/requires_grant` (D1 truth table);
  - `TaskCall::validate()` (§3.3 conditional requirements, caps);
  - `TaskCall::execute()/execute_with()` dispatching into `ledger::`;
  - `parse_args()/TaskArgsOwned` strict wire typing (schema
    violations ⇒ `-32602`; semantic refusals stay in-gate);
  - `tasks_dir()` delegates to the repaired shared resolver.
- **`aiosh-core/src/ledger.rs`**:
  - **D3**: `tasks_dir() -> Result<PathBuf,String>` with ancestor-walk
    for `docs/tasks/MASTER_TASK_LEDGER.jsonl` +
    `find_ancestor_tasks_dir()` helper; `paths() -> Result<…>`;
    CLI `cmd_task` resolves inside the audited closure.
  - **D4**: `rebuild_state` replay — completed⇒next=t+1,
    unblocked⇒next=t, pointer_reset⇒next=t+1, clamp past total⇒None.
- **`aiosh-mcp/src/main.rs`** — `aios.task` registered (manifest
  **12 → 13**, deterministic order); `call_task()` routes through
  `dispatch::recorded_call` with `require_grant = action.requires_grant()`,
  actor `agent:mcp@aiosh-mcp`, one audit row for every outcome;
  bare payloads (`status/check/rebuild`) wrapped into the §3.2
  envelope; schema-violating envelopes answered `-32602` before any
  gate/audit interaction.
- **`ci/rust_smoke.sh`** — count assertion 13; wanted-list gains
  `aios.task`; two new wire cases: `status` (ok, isError:false) and
  `done` without grant (refused at the PEP gate, honest row).

### Python reference (cross-substrate parity, D4)

- `tools/task_ledger.py::rebuild_state` — identical replay semantics.
- `tools/test_task_ledger.py` — U14 (skip/unblock replay), U15
  (end-of-ledger clamp); U8 expectation corrected (see above).
- Two pre-existing test-cfg warnings in `retention.rs` fixed
  (`unused mut`, unused loop var) to hold the zero-warning bar.

## Verification

```
$ cargo build   → 0 errors, 0 warnings
$ cargo test    → test result: ok. 62 passed; 0 failed   (was 52)
$ python3 tools/test_task_ledger.py        → PASS: … (U1..U15)
$ python3 tools/test_task_ledger_scaffold.py → PASS
$ bash code/aiosh-rust/ci/rust_smoke.sh
wire ok: server=aiosh-mcp tools=13 verify_audit_id=49 \
         task_status_next=24 task_refusal_audit=51
parity ok: python read rust-written state (next_task=25)
parity ok: rust read python-written state (next_task=25 blocked=[25])
== RUST SMOKE SUITE PASS ==
$ bash ci/run_all_smokes.sh  → == ALL 10 SMOKE SUITES PASS ==
```

**D3 proof (live):** from an arbitrary cwd with NO `AIOSH_TASKS_DIR`,
`aiosh task check` now resolves `<repo>/docs/tasks` via the ancestor
walk and reports the invariant-clean ledger (`total_tasks: 10000`)
— previously this exact invocation stat-failed at the wrong path
(SPEC-TASK-LEDGER L2). The env override remains highest priority;
Python's file-relative default is untouched.

**Audit effects verified:** refusal wire case produced audit_id 51 with
`gate:"pep"`, `reason:"tool 'aios.task' requires explicit PEP grant"`.

## Acceptance check

- [x] Targeted tests pass (new Rust service/resolver/replay tests;
      Python U14/U15; two new MCP wire cases).
- [x] No regression: full baseline CI **10/10 PASS** after the change.
- [x] No new dependencies.
- [x] Audit/PEP invariants: consequential actions require grant and
      write exactly one audit row each (incl. refusals).
- [ ] *Deferred by design:* SPEC-TASK-LEDGER §7 L2/L3 text updates and
      operator docs land in T-00029 (Documentation), per spec §6.
