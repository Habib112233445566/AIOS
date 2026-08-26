# T-00020 — Task Ledger Control data model: Verification & Evidence

**Date:** 2026-08-22
**Type:** verification & evidence (no code changed)
**Depends on:** T-00019 (complete)
**Environment:** fresh VM, rustup stable 1.98.0 (official installer),
Python 3.13.15, Node 20.19.0. All suites run sequentially per
`ci/run_all_smokes.sh` contract.

> Naming note: this file follows the task *instruction* path
> (`T-00020-verify.md`); the ledger row's `artifacts` field declares a
> truncated variant name (`T-00020-data-model-verification-evidenc.md`),
> which is mirrored byte-for-byte so the declared artifact exists too.

## 1. Epic-specific suites (Task Ledger Control)

### 1.1 Rust unit tests (`aiosh_core::ledger`) — 7/7 PASS

```
$ cargo test ledger::   (code/aiosh-rust)
running 7 tests
test ledger::tests::complete_advances_pointer_exactly_one ... ok
test ledger::tests::block_unblock_skip_flow ... ok
test ledger::tests::invariants_check_passes_and_detects_gaps ... ok
test ledger::tests::no_skip_rejects_out_of_order ... ok
test ledger::tests::events_size_cap_rejects_oversized_log ... ok
test ledger::tests::rebuild_recomputes_from_events ... ok
test ledger::tests::stale_tmp_files_are_cleaned_on_save ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 45 filtered out
```

### 1.2 Python legacy-reference suites — PASS

```
$ python3 tools/test_task_ledger.py
[✓] U10 ledger invariants (good passes, tampered fails)
[✓] U11 no leftover tmp files after atomic saves
[✓] U12 CLI legacy mode, NO-SKIP exit code, check, status
[✓] U13 missing-task lookup refused after no-skip guard
PASS: task ledger unit tests (U1..U13)

$ python3 tools/test_task_ledger_scaffold.py
PASS: task_ledger scaffold — all interfaces present
```

## 2. Full baseline smoke set — 10/10 PASS

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke
PASS: classifier_smoke
PASS: mcp_smoke
PASS: pentest_smoke
PASS: sandbox_smoke
PASS: retention_smoke
PASS: demo_smoke
PASS: cli_bash_smoke
PASS: task_ledger_unit
PASS: task_ledger_scaffold
== ALL 10 SMOKE SUITES PASS ==
```

rust_smoke internals (from `/tmp/aiosh-ci-rust_smoke.log`):

```
test result: ok. 52 passed; 0 failed          # full cargo test suite
wire ok: server=aiosh-mcp tools=12 verify_audit_id=27
cli ok: version 0.1.0
ts-cli run ok: sandbox via {'no_new_privs': 'ok', 'seccomp': 'ok',
 'landlock': 'FAIL: landlock not supported by kernel (pre-5.13?)'}
parity ok: python read rust-written state (next_task=21)
parity ok: rust read python-written state (next_task=21 blocked=[21])
```

Honesty note: `landlock FAIL` is the **expected** host-kernel outcome on
this VM (Colab kernel < 5.13) and the suite correctly reports it as a
component-level failure while seccomp+no_new_privs still enforce — same
fail-open-with-audit behavior documented in the Sprint-2 sandbox notes;
not a regression. Cross-substrate parity ran at live pointer 21 on a
scratch copy and passed in BOTH directions.

## 3. State-file verification

```
$ AIOSH_TASKS_DIR=$PWD/docs/tasks aiosh task check
{"data":{"ok":true,"total_tasks":10000},"ok":true,"subcommand":"task check"}

pre-completion pointer : next_task = 20, completed = 1..19, seq = 19
post-completion pointer: next_task = 21, completed = 1..20, seq = 20
```

Pointer advanced by exactly one via the Rust shipping surface
(`aiosh task done 20 --note … --evidence …`), event appended to
`COMPLETIONS.jsonl`, atomic state rewrite verified by `task status`.

## 4. Milestone

With T-00020 green, the **Task Ledger Control epic (T-00011..T-00020)
is fully closed**: research → specification → implementation (Rust) →
CLI integration → audit-ring wiring → security review → hardening →
operator documentation → verification. `task_plan.md` and
`progress.md` updated accordingly.
