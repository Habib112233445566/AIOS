# T-00084 — Task Ledger Control observability: Implementation

**Date:** 2026-08-22
**Type:** implementation (Rust + Python parity)
**Depends on:** T-00083 scaffold (from concurrent session — see incident note)
**⚠️ CONCURRENT-SESSION INCIDENT:** while this task was in progress, a
second agent session completed T-81..T-83 with a DIFFERENT design
(`metrics` action vs my drafted `health` subcommand) and its scaffold
landed in the shared tree with duplicate `impl` blocks that broke the
build. Resolution: their LOCKED spec (T-00082) governs; I merged their
design cleanly (removed duplicate impls, relocated `build_metrics`
onto `TaskCall` where the call-site expected it), implemented the
remainder per their spec, and reconciled the clobbered research doc.
My superseded `health`-subcommand draft is preserved as an addendum in
T-00081-research.md.

## What shipped

Per locked spec T-00082 — read-only **`metrics` action on `aios.task`**
+ CLI mirror:

- **CLI**: `aiosh task metrics` — consolidated envelope: tasks
  {total/completed/blocked/skipped/next/seq/last_at}, audit
  {rows, verify_ok, head_hash_prefix(12)}, config{6 knobs}. Audited.
- **Rust MCP**: `aios.task metrics` action — gate → compose from THIS
  server's ring (rows via tail-count, verify_ok, head prefix) + ledger
  state + config; one commit row; error paths audited.
- **Python parity**: `_task_metrics()` using `audit_lib.open_db/
  tail-count/verify_live/head_hash`; identical key set; env knobs read
  live (mirrors Rust LedgerConfig defaults).
- Stable ADDITIVE-ONLY key promise recorded on build_metrics docstring.

## Verification (live)

```
$ aiosh task metrics            → ok:true; tasks.next=84; audit.rows=379;
                                  verify_ok=true; config 6 knobs
$ rust-mcp wire metrics         → identical keys; rows tracked live
$ python aios_task(metrics)     → identical keys; rows=381; verify_ok=true
$ cargo test → 79 passed; 0 warnings ; full CI 16/16 PASS
```

## Acceptance check
- [x] Read-only; no grant required (per spec D-table); audited once.
- [x] Parity: same stable keys across CLI/Rust-MCP/Py-MCP.
