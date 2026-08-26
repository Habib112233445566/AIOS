# T-00037 — Task Ledger Control CLI surface: Security Review

**Date:** 2026-08-22
**Type:** security review (no code changed)
**Depends on:** T-00036 integration
**Scope:** the unified `aiosh task` CLI path (`cmd_task` →
`parse_task_args` → `TaskCall::validate/execute_with`), shared
machinery already reviewed in T-00027.

All findings verified empirically on a scratch sandbox (real binary);
nothing asserted from code reading alone.

## 1. Verified controls

| # | Control | Empirical result |
|---|---|---|
| S1 | **Honest audit for every refusal.** 6 distinct usage/semantic refusals (bare `task`, non-numeric id, id=0, not-current block, unknown subcommand, unknown option) each produced exactly one `task.ledger / refused` row. | PASS (6/6 observed) |
| S2 | **Content containment.** Note containing quotes, real newline/tab, backslash, `<script>`, unicode stored VERBATIM-but-JSON-escaped; round-trips byte-exact from `COMPLETIONS.jsonl`. No shell/SQL interpolation anywhere on the path. | PASS |
| S3 | **Evidence-field traversal inertness.** `--evidence ../../etc/passwd` stored as data only; evidence dir received solely the task-id-named stub. | PASS |
| S4 | **Flood caps.** 20 `--evidence` flags → refused at item cap ("exceeds 16 items"); >4096-byte texts refused; single-operand grammar rejects extra operands. | PASS |
| S5 | **Ledger integrity after abuse.** Post-battery `task check` invariant-clean; audit ring `verify_ok: true`. | PASS |
| S6 | **Delimiter hardening closes the old injection-ish quirk.** Pre-T-34, `--reason --note` swallowed a flag as a value (T-00031 F6). Now refused with an explicit message; literal dash-values require the deliberate `--` form and are stored verbatim (C8). | PASS |

## 2. Abuse scenarios → dispositions

| Scenario | Disposition |
|---|---|
| Option-like payloads smuggled as values (`--reason --force`) | Refused pre-execution (S1-class usage refusal, audited); deliberate literals via `--` stored verbatim but inert (S2) |
| Newline/control chars laundering into the JSONL event log | json escaping keeps one-event-per-line invariant (round-trip proven) |
| Env redirect (`AIOSH_TASKS_DIR`) to attacker-chosen dir | Operator trust boundary, unchanged from data-model epic; agent cannot set server env |
| DoS via argv flood | Item/count caps + OS ARG_MAX bound; refusals audited |
| Post-`--` option smuggling | After `--`, ALL tokens are values/literals by definition — options cannot be reactivated mid-line (G10); documented |

## 3. Non-blocking notes

- Refusal envelope prints to stderr (`err_out` convention) while
  success prints to stdout — intentional Unix split, now pinned by C1–C9.
- Residual: output-side PI scanning of ledger content remains the
  separately-tracked `scan_output_for_pi` proposal (as in T-00027).

## 4. Verdict

**No known policy bypass remains open** on the CLI surface.

Acceptance:
- [x] Security evidence file with abuse scenarios.
- [x] No known policy bypass remains open.
