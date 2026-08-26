# T-00087 — observability: Security Review (evidence)

**Date:** 2026-08-22 · **Scope:** `metrics` action on all three surfaces
(Rust MCP wire / Rust CLI / Python reference tool)
**Verdict:** **NO policy bypass found.** All probes empirical (commands +
outputs below); nothing assumed.

## S1 — PEP gating & grant truth table
- `metrics` WITHOUT a grant → allowed (read-only by design; matches spec D1:
  `requires_grant()==false`). Audited: `audit_id 1`.
- Mutation regression check on the same wire session:
  `{action:"done",task_id:1,note:"x"}` without grant →
  `ok:false, gate:"pep", audit_id 2`. **Gate intact.**

## S2 — Hostile state file (input validation / DoS)
| Payload | Result |
|---|---|
| 10 MiB junk string (>4 MiB cap) | `{"error":"… too large (10485760 bytes > cap 4194304 bytes)","ok":false}`, exit 1, process alive |
| 300-level nested JSON (> serde_json 128 depth) | `"recursion limit exceeded at line 1 column 1779"`, ok:false, exit 1 |
| Truncated JSON (`{broken`) | loud parse error naming position; ok:false |

No panic path found (exit code never 101/134); every hostile input yields the
standard envelope. Size caps bound memory BEFORE parse (`read_capped`).

## S3/S5 — Path & argument injection
- `aios.task` schema exposes only `action/task_id/note/reason/evidence/grant_id`
  (verified via tools/list). None reaches the filesystem for `metrics`;
  ledger location comes exclusively from operator env
  (`AIOSH_TASKS_DIR`), which agents cannot set over MCP.

## S4 — Env-knob abuse (operator surface)
`AIOSH_LEDGER_MAX_TEXT=99999999999999999999999` →
`"invalid AIOSH_LEDGER_MAX_TEXT='…': not a decimal integer"` — loud named
refusal, audited envelope, no overflow/panic. Knobs are deliberately NOT
agent-reachable over MCP (SPEC §8.3).

## S6 — Audit-row emission (one honest row per call)
Fresh DB probe:
```
before 0 → ok-call → (+1) → refusal-call (task metrics 7) → (+1)
```
(The intermediate +2 appearance is the measuring `audit tail` writing its own
row.) Both ok AND refused metrics calls extend the chain — ADR-0035 §F-2 /
SPEC §8 satisfied post-T-00085 fixes.

## S8 — Information disclosure
`data.tasks` carries COUNTS ONLY (`completed/blocked/skipped` are ints;
verified empirically). No task titles, ids, notes, evidence paths, or note
contents are exposed. `audit.head_hash_prefix` (12 hex) is public chain
metadata. Config values are the operator's own knob settings.

## Residual risks (documented, not bypasses)
- R1: `metrics` runs `ring.verify()` per call — O(live rows). Bounded by
  retention (`keep_rows`); noted as cost, not a vuln.
- R2: Python pre-gate validation failures return WITHOUT an audit row
  (pre-gate by design, documented in SPEC §8.2); Rust refuses WITH a row.
  Envelope parity asserted in T-00085 O-suite.

**Blocking notes:** none.
