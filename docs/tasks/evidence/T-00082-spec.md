# T-00082 — Task Ledger Control observability: Specification

**Date:** 2026-08-22
**Type:** specification (no code changed)
**Depends on:** T-00081 research
**Status:** SPECIFIED — D1–D5 locked to defaults (standing autonomy).

## 1. Contract: `metrics` action

New READ-ONLY action on the existing grouped tool + CLI mirror:

- MCP: `aios.task {"action":"metrics"}` — `require_grant=false`,
  audited like every call; inputSchema enum grows by one value.
- CLI: `aiosh task metrics` — same snapshot, same envelope.

## 2. Stable key set (additive-only evolution promise, D4)

```json
{ "ok": true, "action": "metrics",
  "tasks": { "total_tasks": n, "completed": n, "blocked": n,
              "skipped": n, "next_task": n|null,
              "last_event_seq": n, "last_completed_at": ts|null },
  "audit": { "rows": n, "verify_ok": bool,
             "head_hash_prefix": "<16 hex>" },
  "config": { "lock_timeout_secs": n, "max_ledger_bytes": n,
               "max_events_bytes": n, "max_state_bytes": n,
               "max_text": n, "max_evidence_items": n } }
```

Sources (all reused): `ledger::load_state`, `AuditRing::verify` +
`tail(1)`, `LedgerConfig::from_env`. Light LIVE verify only (A1/D3);
full replay stays on `aios.audit.verify(full=true)`.

## 3. Cross-substrate parity (D5)

Python reference server implements the same action via
`_run_task_action` extension (`metrics`) composing the identical keys
from `load_state()` + audit_client tail/verify + `_env_int` knobs.
Both substrates' outputs must match key-for-key (asserted in new tests).

## 4. Failure matrix

State/corrupt-file errors surface as today (`ok:false`, audited);
`next_task` may be null (end-of-ledger) — monitors treat null as
"project complete", not an error. No other failure modes introduced;
read-only ⇒ no grant, zero persistence beyond the one audit row.

## 5. Out of scope
HTTP/Prometheus endpoints; historical time-series; log shipping.

## 6. Reviewability check
Key set §2; sources §2; parity §3; failures §4 — standalone reviewable.
