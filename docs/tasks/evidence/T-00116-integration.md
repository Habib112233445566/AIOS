# T-00116 — CI Smoke Orchestration / data model: Integration

Date: 2026-08-23 · Status: INTEGRATION COMPLETE — the registry is now the
single source of CI truth; full CI runs through it end-to-end.

## Wiring changes

| File | Change |
|---|---|
| `tools/ci_run.py` (NEW) | production orchestrator: iterates `ci_suites.SUITES` sequentially via `subprocess.run` with per-suite **timeouts** (closes gap G5), identical human output (`==> [name] starting`, `PASS:/FAIL:` + 40-line tail, final ALL-PASS banner), same `/tmp/aiosh-ci-<name>.log` paths, ports the bash host-repair (`chmod +x node_modules/.bin/*`) and `$PYTHON` override, fail-fast preserved, writes the atomic `RunSummary` artifact (`$AIOSH_CI_RESULTS`, default `/tmp/aiosh-ci-results.json`) |
| `ci/run_all_smokes.sh` | becomes a thin delegating shim (`exec python3 tools/ci_run.py`), keeping the historical entrypoint + sequential-contract header. Suite list no longer duplicated in bash |

## End-to-end proof (the integrated path IS production)

```
AIOSH_CI_RESULTS=/tmp/opencode/ci-summary-live.json bash ci/run_all_smokes.sh
→ exit 0, "== ALL 19 SMOKE SUITES PASS (179788 ms) =="
summary: total=19 passed=19 all_pass=true,
order rust_smoke → task_docs_scaffold preserved,
slowest suite visible in data (demo_smoke, 40599 ms)
```

## Bugs caught by integration (honest log)

1. First live run crashed after suite 1: ci_run passed its message-detail
   variable (`None`) as `exit_code` on the pass path — the T-00114
   validator rejected it exactly as designed ("exit_code must be an int
   for pass/fail records"). Fixed to pass `proc.returncode`.
2. W-suite W1 then failed BY DESIGN: it asserted registry==bash-file
   order, but the bash list was intentionally deleted. W1 rewritten to
   pin a frozen canonical-order tuple (19 names) + shim delegation —
   the single-source flip is now itself regression-pinned.

## Known limitation recorded

Timeout kills the direct child only; grandchildren may linger (legacy had
no timeout at all). Summary honestly records status="timeout".

## Verification

- Full CI through new path: 19/19 PASS (exit 0) with summary artifact.
- W-suite W1..W7 green post-flip; docs checker C1..C6 green.
