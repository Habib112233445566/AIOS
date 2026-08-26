# T-00048 — Task Ledger Control MCP/API surface: Hardening

**Date:** 2026-08-22
**Type:** hardening (Python reference surface)
**Depends on:** T-00047 security review

## Gaps found and fixed (each proven by probe)

1. **Module re-import per call.** `_load_task_ledger()` re-executed
   `tools/task_ledger.py` on EVERY invocation — repeated disk reads,
   N separate module instances. Now cached in `_TASK_LEDGER_MOD`
   (identity-probe verified).
2. **Unguarded loader.** The loader sat OUTSIDE the business try/except:
   a missing/corrupt module after a gate pass would leak a framework
   exception with NO honest audit row (ADR-0035 §F-2 violation). Moved
   inside the guarded section; loader failure now yields a clean
   `{ok:false, action, error}` envelope + committed error row
   (verified against `/nonexistent` dir: `[Errno 2]` envelope observed).
3. **bool task_id coercion edge.** Python `True == 1` let
   `task_id=True` address task 1 even WITH a grant. Explicit
   `isinstance(task_id, bool)` rejection added to validation; probe:
   refused with-grant ("'task_id' must be a positive integer >= 1").

## Verified-not-added

Lock wait bounded (T-28 shared); caps enforced pre-gate (T-45 suite);
FastMCP owns protocol-layer type validation for wire clients; no child
processes/temp files on this path; error paths commit rows.

## Verification

```
H1 cache identity · H2/H3 probes above → all green
P1..P8 suite re-run → PASS
bash ci/run_all_smokes.sh → == ALL 13 SMOKE SUITES PASS ==
```

## Acceptance check
- [x] Failure modes explicit + auditable (loader-failure row now exists).
- [x] No temp/connection leaks (re-audited; none on this path).
