# T-00035 — Task Ledger Control CLI surface: Unit Test

**Date:** 2026-08-22
**Type:** unit/wire tests
**Depends on:** T-00034 implementation

## What shipped

`code/aiosh-cli/tests/test_task_cli_smoke.py` — standalone suite in the
repo smoke style driving the REAL `aiosh` binary in an isolated sandbox
(temp `AIOSH_TASKS_DIR`/`AIOSH_HOME`), asserting observable behavior:
stdout/stderr envelopes per the CLI's established stream convention
(success→stdout via ok_out, refusal→stderr via err_out — harness parses
both), exit codes, ledger files, event log, and audit-ring rows read
back through `aiosh audit tail`.

| Case | Class | Asserts |
|---|---|---|
| C1 | valid | status envelope, exit 0, label "task status" |
| C2 | valid | done: pointer+event+evidence stub + audit row (tool task.ledger, outcome ok) |
| C3 | invalid | empty --note refused; state file byte-identical; no new events |
| C4 | invalid | missing value at end → "missing value for '--reason'" |
| C5 | boundary | note 4097 bytes refused ("exceeds") |
| C6 | boundary | task_id 0 refused ("must be >= 1") |
| C7 | failure mode | NO-SKIP refusal: exit 1, unchanged state/events, audited row outcome=refused |
| C8 | boundary | `--reason -- --weird-reason` stores literal dash-leading reason in envelope AND event |
| C9 | valid | help exits 0 with zero side effects |

Also covered at unit level inside `main.rs` (T-00034): 13 Rust tests
for parse_task_args/task_usage_text incl. dash-value rejection,
unknown-option naming, delimiter-in-value-position, extra-operand.

## Harness honesty

First isolated run crashed at C3: refusals go to STDERR (err_out
convention I had not accounted for). Fixed the HARNESS to parse both
streams — no product change. Static LSP notes on loose json.loads
typing match the repo's existing suites' style; runtime is authoritative.

## Verification

```
$ python3 code/aiosh-cli/tests/test_task_cli_smoke.py
[✓] C1..C9   PASS: task cli wire smoke (C1..C9)
$ broken-check: sabotaged expectation copy → [✗] W1-style failure, exit 1
$ cargo test (workspace) → 77 passed; 0 failed
```

## Acceptance check

- [x] New test file runs standalone and passes.
- [x] Negative cases asserted (C3/C4/C5/C6/C7), not just happy path.
