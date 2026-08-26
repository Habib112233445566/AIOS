# T-00027 — Task Ledger Control core service: Security Review

**Date:** 2026-08-22
**Type:** security review (no code changed; one pre-existing behavior noted)
**Depends on:** T-00026 integration
**Scope:** the `aios.task` MCP surface + `task_service` module
(`aiosh-mcp/src/main.rs::call_task`, `aiosh-core/src/task_service.rs`,
shared `ledger.rs`/`dispatch.rs`/`pep.rs` machinery)

All findings below were **verified empirically** against a scratch
sandbox (temp `AIOSH_TASKS_DIR` + temp `AIOSH_HOME`, real binaries);
nothing is asserted from code reading alone.

---

## 1. Verified controls (with evidence)

| # | Control | Empirical result |
|---|---|---|
| S1 | **PEP gating of every state-changing path.** `done/block/unblock/skip/rebuild` without grant → refused at `gate:"pep"` with an honest audit row each (audit_ids 3–7 observed). Read-only `status/check` need no grant by design (D1). | PASS |
| S2 | **Grant-scope isolation.** A grant scoped `tools=["aios.pentest.*"]` does NOT authorize `aios.task`; refusal reason names the exact mismatch (`tool 'aios.task' not in grant scope.tools=[…]`). | PASS |
| S3 | **Input validation / injection surface.** Strict wire parsing: unknown keys, non-enum actions, wrong types, empty strings, >4096-byte texts, >16 evidence items, `task_id < 1` all rejected `-32602` before any gate or disk interaction (T-00025 W3–W6 + Rust unit tests). No shell invocation anywhere in the path; state/events written via serde; SQLite via parameterized rusqlite. | PASS |
| S4 | **Hostile payload containment.** Note/evidence containing quotes, real newlines/CRLF/tab, backslashes, `../../etc/passwd`, `C:\Windows\system32`, `<script>` and unicode was stored **verbatim but JSON-escaped** in `COMPLETIONS.jsonl` and audit args — parsed back byte-exact; evidence directory received ONLY the task-id-named stub (`T-00001-completion.md`); no file writes outside the sandbox root. | PASS |
| S5 | **Numeric extremes.** `task_id = 2^64−1` with a valid grant → clean NO-SKIP refusal (audited), no overflow/panic; u64 parse rejects negatives/floats at the schema layer. | PASS |
| S6 | **Audit-chain integrity after abuse.** `aiosh status` on the abused ring: `verify_ok: true` — every refusal and mutation extended the chain; nothing truncated it. | PASS |
| S7 | **Classifier coverage.** Every `aios.task` call is classified like any tool (policy_revision recorded on rows; request-side PI scanning applies to args including note/reason text). | PASS (by wire observation) |

## 2. Abuse scenarios considered → dispositions

| Scenario | Disposition |
|---|---|
| Agent advances the queue without human intent (skip/done spree) | Requires a valid grant scoped to `aios.task` (S1/S2); every advance is double-logged (audit row + `COMPLETIONS.jsonl`) with grant attribution — full traceability. Accepted by D1/D7 decision. |
| Model-controlled `note` poisons future readers (indirect prompt injection via ledger files) | Content stored inert (S4); readers reach it through gated surfaces (`fs.read` safe-roots / CLI). Residual risk accepted for this epic: output-side PI scanning (`scan_output_for_pi`, research gap doc §4) is a separately tracked proposal — not a bypass of THIS component's policy. |
| Evidence field used for path traversal | Evidence strings are data only; stub filename derives solely from numeric task id (S4). |
| DoS via giant payloads | Schema caps + T-00018 file-size caps bound memory; flock serializes writers. |
| Grant reuse across tools | Glob-scoped grants; namespace isolation proven (S2). |
| Env-var redirect abuse (`AIOSH_TASKS_DIR`) | Operator-level trust boundary, identical to the pre-existing CLI contract; not agent-reachable (agent cannot set server env). |

## 3. Notes (non-blocking)

- `pep::is_irreversible()` covers `fs.write*`, `pentest.*`, reboot/shutdown —
  `aios.task` mutations are deliberately NOT classed irreversible: they are
  append-only-event operations, fully rebuildable from `COMPLETIONS.jsonl`
  (D4 replay), so no destructive irreversibility exists to gate.
- Harness honesty: two initial script failures during this review were
  bugs in my verification scripts (wrong JSON paths), corrected before
  any conclusion was drawn; product behavior was never implicated.

## 4. Verdict

**No known policy bypass remains open.** All state-changing paths are
grant-gated and audited; input validation is schema-complete;
hostile content is inert; the chain verifies after abuse.

Acceptance:
- [x] Security evidence file exists with abuse scenarios.
- [x] No known policy bypass remains open.
