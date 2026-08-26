# T-00042 — Task Ledger Control MCP/API surface: Specification

**Date:** 2026-08-22
**Type:** specification (no code changed)
**Depends on:** T-00041 research
**Status:** SPECIFIED — D1–D5 locked to research defaults (standing
autonomy granted by project owner, 2026-08-22)

All new behavior is an **AIOS-specific proposal** on the reference
substrate; upstream-facing semantics conform to the MCP Tools spec as
cited in T-00021 (E1–E5).

## 1. Resolved decisions

| ID | Decision |
|---|---|
| D1 | Full 7-action mirror on the Python reference server: one FastMCP tool `aios_task` with `action` argument (status/check/done/block/unblock/skip/rebuild) |
| D2 | Gate/audit tool string = **`"aios.task"`** (matches Rust) so a single `--tools "aios.task"` grant authorizes both servers; audit rows carry `command="task.<action>"` |
| D3 | Server-side validation before the gate, mirroring Rust: non-empty note/reason where required, text ≤4096 bytes, evidence ≤16 items each ≤4096; violations → normal result `{ok:false, error:…}` (FastMCP maps exceptions; we return envelopes like rotate does) |
| D4 | Envelope parity with Rust: `{ok, action, data|error, audit_id?, classifier_policy_revision?}`; bare payloads (`status/check/rebuild`) wrapped as `{"ok":true,"action":…,"data":…}` |
| D5 | CI: `test_smoke.py` expected-set gains `aios_task`; new wire-level checks appended there (status ok + done-without-grant refusal); `rust_smoke.sh` untouched |

## 2. Contract

```
aios_task(action: str, task_id: int|None = None, note: str|None = None,
          reason: str|None = None, evidence: list[str]|None = None,
          grant_id: str|None = None) -> dict
```

- Actions/grant policy: identical table to Rust (SPEC-TASK-LEDGER §8):
  status/check read-only; done/block/unblock/skip/rebuild require a
  PEP grant whose scope covers `"aios.task"`.
- Validation order mirrors Rust: structural/type + caps first
  (`ok:false`, no gate interaction), then conditional-presence via
  shared validate-equivalent, THEN gate → exactly one audit row per
  call regardless of outcome.
- Ledger operations delegate to `tools/task_ledger.py` functions
  (module imported once; paths re-bound from `AIOSH_TASKS_DIR` at
  import time — server inherits operator env), preserving no-skip,
  atomic state writes, fsync'd events, bounded lock wait.
- Actor: `agent:mcp@aiosh-mcp` (same as pentest wrappers).

## 3. Reused vs new

Reused: `_dispatch.dispatch/commit`, `task_ledger.*`,
`active_constitution_rev`. New: one decorated tool fn + small
validation helper in `server.py`; smoke additions. No new deps.

## 4. Failure matrix (mirrors Rust §3.3)

unknown action / missing id / empty-or-missing note-reason / oversize
/ NO-SKIP / not-current / corrupt files / lock-busy → all
`{ok:false,…}` results; refusals and errors additionally audited;
success audited. No silent paths.

## 5. Out of scope
Renaming existing Python tools (naming divergence documented, F5/A1);
HTTP APIs; Resources/Prompts.

## 6. Reviewability check
Happy path §2; failures §4; audit effects §2; reused/new §3 —
reviewable without reading implementation.
