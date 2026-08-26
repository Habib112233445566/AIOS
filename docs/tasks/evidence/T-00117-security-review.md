# T-00117 — CI Smoke Orchestration / data model: Security Review

Date: 2026-08-23 · Status: REVIEW COMPLETE — no open policy bypass.

## Attack-surface analysis + empirical probes

| ID | Scenario | Result |
|---|---|---|
| S1 | Shell/argument injection via suite commands | commands are argv LISTS executed with `subprocess.run(..., shell=False)`; grep confirms zero `shell=True` / `os.system` / `popen` in `tools/ci_run.py` + `tools/ci_suites.py` ✅ |
| S2 | Predictable-temp symlink attack on summary writer (`<target>.tmp.<pid>` in world-writable /tmp) | **direct probe**: attacker pre-places a SYMLINK at the exact tmp path of the writing process → `O_EXCL` open fails loudly (`FileExistsError`), `/etc/passwd` untouched, target never clobbered ✅ |
| S3 | Stale temp from a dead pid (different pid) | write proceeds to its own O_EXCL temp; stale file never followed or overwritten ✅ |
| S4 | Hung/malicious suite (DoS) | wall-clock timeout enforced — direct child killed at deadline (probe: 30 s sleeper killed at 1.0 s). Documented residual: grandchildren may survive (legacy bash had NO timeout; strict improvement) |
| S5 | Log-path manipulation via suite names | names are registry-controlled static data, not user input; log paths derive solely from them ✅ |
| S6 | `AIOSH_CI_RESULTS` abuse | operator knob by design (same trust class as CI output dirs); writes go through the hardened atomic writer (S2/S3); JSON-only content, consumers use json.load (no eval) |
| S7 | Ledger/audit interaction | orchestrator is host-side tooling per spec §6: imports neither `task_ledger` nor audit modules; touches nothing under docs/tasks (verified by import graph) |

## Input validation

Registry validated AT IMPORT (duplicate names, malformed commands, bad
timeouts — pinned by W7). Constructors reject unknown suites,
index-position mismatch (order-is-contract), bad statuses, negative
durations, non-Z timestamps — all field-naming (W-suite).

## Verdict

Fail-closed where it matters (temp collisions, invalid data at load),
no shell interpretation, no untrusted input reaches paths or commands.
Residual risks are documented limitations (grandchild processes after
timeout), not policy bypasses.
