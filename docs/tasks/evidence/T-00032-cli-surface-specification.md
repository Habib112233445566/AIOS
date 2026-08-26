# T-00032 — Task Ledger Control CLI surface: Specification

**Date:** 2026-08-22
**Type:** specification (no code changed)
**Depends on:** T-00031 research (`T-00031-research.md`)
**Status:** SPECIFIED — decisions D1–D5 locked by the project owner
(2026-08-22, "yeah lock these in")

> All grammar/validation choices below are **AIOS-specific proposals**
> layered on the existing `aiosh task` command; external behavior
> follows POSIX.1-2024 XBD ch.12 guidelines where cited (E1–E6 in
> research).

---

## 1. Resolved decisions (owner-locked)

| ID | Decision |
|---|---|
| D1 | **Unify validation**: `cmd_task` routes through `task_service::TaskCall` (`parse_args`-equivalent + `validate` + `execute_with`). Empty `--note` on `done` becomes a refusal, matching MCP. Zero measured compat breakage (research §data: 31/31 historical notes non-empty; no bare-`done` callers). |
| D2 | Reject dash-leading flag values explicitly; support `--` end-of-options delimiter (POSIX G10/G14). |
| D3 | Add `aiosh task help`; fix `"task "` trailing-space label. |
| D4 | Enforce evidence caps on CLI: maxItems 16, item ≤4096 bytes (schema parity). |
| D5 | Options-after-operands grammar stays (GNU-style deviation from POSIX G9), now **documented as intentional** in help text. |

## 2. Command contract

```
aiosh task <subcommand> [operand] [options]

Subcommands (unchanged set):
  status                       print TASK_STATE.json envelope
  check                        validate ledger invariants
  done     <task_id> --note <text> [--evidence <path>]... 
  block    <task_id> --reason <text>
  unblock  <task_id> --reason <text>
  skip     <task_id> --reason <text>
  rebuild                      recompute pointer from event log
  help                         per-subcommand usage text (new)

Grammar rules:
  - <task_id>: decimal u64 >= 1; parse failure = usage error (E6).
  - Option values MUST be separate arguments and MUST NOT start with
    "--" (D2); a value-looking flag → usage error naming the option.
    A missing value at end-of-line → usage error (POSIX G7).
  - "--" ends option parsing; following tokens are operands/values
    even if dash-prefixed (D2/G10). Needed to legitimately pass
    dash-leading note text.
  - Options may appear after the operand (documented deviation, D5/G9).
```

### 2.1 Validation semantics (single source: `task_service`)

CLI builds a `TaskArgsOwned`-shaped struct from argv, then calls the
same `validate()` + `execute_with(&paths)` used by the MCP tool:

| rule | CLI behavior (NEW) | was |
|---|---|---|
| `done` requires non-empty `--note` | usage error | silently stored "" |
| text flags ≤4096 bytes | usage error "exceeds 4096" | uncapped |
| evidence ≤16 items, each ≤4096 | usage error | unbounded |
| `--reason`/`--note` value starting with `--` | usage error | consumed as value |
| unknown flag tokens among values | usage error naming token | swallowed |

Error text convention: `"usage: aiosh task done <task_id> --note <non-empty text up to 4096 bytes>"` etc.

### 2.2 Outputs, exit codes, persistence (unchanged)

- Success: `{ok:true, subcommand:"task <sub>", data:{…}}`, exit 0.
- Failure: `{ok:false, …, error:<reason>}`, exit 1 — includes all
  validation refusals above.
- Top-level unknown command remains exit 2 (outside this component).
- Persistence effects identical to today: underlying `ledger::` calls;
  every outcome (ok/refused/usage-error) writes exactly one honest
  `task.ledger` audit row (F2/ADR-0035 §F-2 — unchanged).
- `help` subcommand: exit 0, no audit row (pure documentation read,
  like `--help` at top level today).

## 3. Interfaces

**Reused verbatim:** `task_service::{TaskAction, TaskCall,
TaskArgsOwned-shaping, validate, execute_with}`, `ledger::paths()`,
audit `emit`, `flag_after` (replaced by a stricter `parse_flags`
helper inside cmd_task).

**New:** `aiosh-cli` local helper `parse_task_args(argv) ->
Result<TaskCall, String>` translating argv → owned args (the mirror of
`task_service::parse_args` for non-JSON input), plus `help` text table.
No new dependencies. No changes to `aiosh-core`.

## 4. Failure-path matrix

| Condition | Result |
|---|---|
| missing subcommand / unknown subcommand | usage error, exit 1, audited |
| non-numeric or `< 1` task_id | usage error, exit 1, audited |
| empty/missing `--note` (done) | usage error, exit 1, audited |
| missing/empty `--reason` (block/unblock/skip) | usage error, exit 1, audited (ledger guard unchanged underneath) |
| oversized note/reason/evidence | usage error, exit 1, audited |
| dash-leading option value without `--` | usage error, exit 1, audited |
| NO-SKIP / not-current-id / lock-busy / corrupt-file | unchanged runtime refusals, exit 1, audited |

## 5. Out of scope

Changing the MCP surface (already stricter), reordering grammar to
options-first (D5 declines), renaming subcommands, JSON-output flags
(output is already JSON).

## 6. Reviewability check

Happy path: §2. Failure paths: §4 matrix. Audit effects: §2.2.
Reused vs new: §3. A reviewer can validate this contract without
opening the implementation.
