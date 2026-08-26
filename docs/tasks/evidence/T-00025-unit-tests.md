# T-00025 — Task Ledger Control core service: Unit Test

**Date:** 2026-08-22
**Type:** unit/wire tests (one production-code boundary fix)
**Depends on:** T-00024 implementation

## What shipped

**New standalone suite** `code/aiosh-mcp/tests/test_task_service_smoke.py`
(repo smoke style: PASS/FAIL markers, non-zero exit on failure) driving
the REAL `aiosh-mcp` binary over stdio JSON-RPC in a fully isolated
sandbox (temp `AIOSH_TASKS_DIR` + temp `AIOSH_HOME`; PEP grant minted
against the same sandboxed audit DB). Asserts observable behavior only
— envelopes, ledger files, event log, evidence stub, audit ids.

| Case | Class | Asserts |
|---|---|---|
| W1 | valid | status envelope: `ok/action/data.next_task/audit_id` |
| W2 | valid | done (with grant): pointer advance + `COMPLETIONS.jsonl` event appended + evidence stub file exists |
| W3 | invalid | unknown enum action → JSON-RPC `-32602` |
| W4 | invalid | unexpected argument key → `-32602` |
| W5 | boundary | `task_id: 0` → `-32602` (schema minimum 1) |
| W6 | boundary | note > 4096 bytes → `-32602` (schema maxLength) |
| W7 | failure mode | NO-SKIP refusal: `isError:true`, byte-identical state file, zero events added, refusal audited |
| W8 | gate refusal | mutation without grant → `gate:"pep"`, exact reason string, honest audit row, zero state change |

## Boundary-driven production fix (found while writing W5)

`parse_args` accepted `task_id: 0` although the declared inputSchema is
`minimum: 1`. Fixed in `task_service.rs::parse_args` (`>= 1` enforced,
protocol-level rejection), plus Rust unit coverage:
`parse_args_strict_types` extended; new `parse_args_schema_bounds`
(minLength/maxLength/maxItems/minimum are schema constraints ⇒ `-32602`,
while conditional *presence* rules stay gate-level refusals).

Also caught a stale-binary trap honestly: the first W5 run failed until
`cargo build` refreshed the MCP binary — suite verified against current
code only after rebuild.

## Broken-feature check (template requirement)

Sabotaged copy of the suite (expectation flipped to `next_task == 999`)
run from the same directory:

```
[✗] W1 status envelope (ok/action/data/audit_id)
exit=1
```

Real suite immediately after: `PASS: task service wire smoke (W1..W8)`.

## Verification

```
$ python3 code/aiosh-mcp/tests/test_task_service_smoke.py   → PASS (W1..W8)
$ cargo test -p aiosh-core   → ok. 63 passed; 0 failed      (zero warnings)
$ bash ci/run_all_smokes.sh  → == ALL 10 SMOKE SUITES PASS ==
```

Note: the new suite runs standalone today; wiring it into
`ci/run_all_smokes.sh` belongs to T-00026 (Integration), matching that
task's "wire into the real call path / registration point" instruction.
