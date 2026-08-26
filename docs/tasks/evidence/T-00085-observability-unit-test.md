# T-00085 — observability: Unit Test (evidence)

**Date:** 2026-08-22 · **Suite:** `code/aiosh-mcp/tests/test_metrics_smoke.py` (new)
**Baseline before work:** `ci/run_all_smokes.sh` → **16/16 PASS**
**Result:** **12/12 checks PASS standalone**; cargo test 79/79 (13 cli + 66 core, zero warnings).

## What the suite pins

| # | Case | Surface |
|---|---|---|
| O1 | valid call → ok + stable keys {tasks,audit,config} + audit_id>0 | Rust MCP wire |
| O2/O2b | parity: identical top-level key set vs wire | Python tool |
| O3 | ok envelope + config == LedgerConfig defaults | CLI |
| O4 | `metrics` WITH `task_id` → refused (isError:true, "does not take") | Rust MCP wire |
| O5 | stray CLI operand (`task metrics 5`) → loud usage refusal, never silent-ok | CLI |
| O6 | task_id refused PRE-GATE without audit_id (reference behavior) | Python |
| O7a/b/c | pristine ring rows==0/next_task==1/head="" ; env override visible (max_text=64); env below floor (63<64) → loud named error | all |
| O8a/b | corrupt TASK_STATE.json → ok:false on BOTH surfaces; CLI still emits one honest audit row (`task.ledger` refused) | both |

## Broken-feature proof & defects caught (tests-first)

The negative cases were written against the SPEC contract first and were run
against the unmodified tree. They FAILED, exposing two real defects:

1. **O4 — Rust MCP wire accepted `{action:"metrics", task_id:7}`** (isError:false).
   The Metrics branch bypassed `TaskCall::validate`, diverging from the Python
   reference (pre-gate refusal). **Fixed** in
   `aiosh-rust/aiosh-mcp/src/main.rs`: metrics now refuses a present `task_id`
   with `"action 'metrics' does not take 'task_id'"` and commits exactly one
   honest `refused` audit row (SPEC-TASK-LEDGER §8 invariant).
2. **O5 — CLI silently ignored a stray operand** (`aiosh task metrics 5`
   returned ok). **Fixed** in `aiosh-rust/aiosh-cli/src/main.rs`: extra tokens
   produce a loud usage-refusal envelope (audited via the standard cmd_task
   emit path).
3. *(Test-side)* O8b initially asserted the wrong audit tool name; the CLI was
   in fact already writing its honest row as `tool="task.ledger"` — assertion
   corrected to match the real contract.

Before/after captured in-session:
- pre-fix wire probe: `id4 isError False … audit_id 2` (accepted);
  post-fix: refused with audited row.
- pre-fix CLI: `task metrics 5` → ok envelope; post-fix → stderr
  `{"ok":false,"error":"unexpected argument '5' — 'metrics' takes no operands"}`.

## Verification commands

```
python3 tests/test_metrics_smoke.py          # 12/12 PASS (standalone)
cargo build && cargo test                    # zero warnings, 79/79 PASS
```

Negative-case discipline: refusals asserted by message content AND envelope
shape, not implementation internals.
