# T-00030 — Task Ledger Control core service: Verification & Evidence

**Date:** 2026-08-22
**Type:** verification & evidence (no code changed)
**Depends on:** T-00029 documentation

> Naming note: instruction path `T-00030-verify.md`; the ledger row's
> declared artifact (`T-00030-core-service-verification-evidenc.md`)
> is mirrored byte-for-byte.

## 1. Epic-specific suites — all PASS

```
$ cargo test   (code/aiosh-rust)
test result: ok. 64 passed; 0 failed          # zero warnings
  incl. task_service parse/validate/execute, resolver ancestor-walk,
        rebuild replay + end-clamp, lock-contention timeout

$ python3 tools/test_task_ledger.py           → PASS: … (U1..U16)
$ python3 tools/test_task_ledger_scaffold.py  → PASS
$ python3 code/aiosh-mcp/tests/test_task_service_smoke.py
                                              → PASS: task service wire smoke (W1..W8)
```

## 2. Full baseline smoke set — 11/11 PASS

```
$ bash ci/run_all_smokes.sh
PASS: rust_smoke            # build + 64 tests + wire contract + 4-way parity
PASS: classifier_smoke
PASS: mcp_smoke
PASS: task_service_smoke    # NEW: core-service wire suite in CI
PASS: pentest_smoke
PASS: sandbox_smoke
PASS: retention_smoke
PASS: demo_smoke
PASS: cli_bash_smoke
PASS: task_ledger_unit      # U1..U16
PASS: task_ledger_scaffold
== ALL 11 SMOKE SUITES PASS ==
```

rust_smoke internals:

```
wire ok: server=aiosh-mcp tools=13 verify_audit_id=100 \
         task_status_next=30 task_refusal_audit=102
parity ok: python read rust-written state (next_task=31)
parity ok: rust read python-written state (next_task=31 blocked=[31])
parity ok: rust rebuilt python-written events (skip replayed, next_task=32)
parity ok: python read rust-written skip (next_task=33 skipped_tail=32)
```

Honesty note (unchanged from T-00020): `landlock FAIL` inside the TS
sandbox line is the expected pre-5.13-kernel outcome on this VM;
seccomp+no_new_privs enforce, fail-open-with-audit by design.

## 3. State-file verification

```
pre-completion pointer : next_task = 30, completed = 1..29, seq = 29
post-completion pointer: next_task = 31, completed = 1..30, seq = 30
aiosh task check → {"ok":true,"total_tasks":10000}
```

Pointer advanced exactly one via `aiosh task done 30` (the shipped
surface under test).

## 4. Milestone — core service sub-epic CLOSED (T-00021..T-00030)

Research → specification → scaffold → implementation → unit tests →
integration → security review → hardening → documentation → this
verification. The agent-facing ledger surface now exists end-to-end:
`aios.task` over MCP behind the classifier→PEP→audit gate, with the
D3 resolver repair, D4 rebuild replay (both substrates), bounded lock
wait, and a 1 MiB transport cap — all documented in
`docs/SPEC-TASK-LEDGER.md` §7–§9 with limitations stated honestly.
`task_plan.md` and `progress.md` updated with the milestone.
