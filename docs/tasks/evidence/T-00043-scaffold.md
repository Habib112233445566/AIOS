# T-00043 — Task Ledger Control MCP/API surface: Scaffold

**Date:** 2026-08-22
**Type:** scaffold (interfaces only; body fails loudly)
**Depends on:** T-00042 spec

## What shipped

`code/aiosh-mcp/aiosh_mcp/server.py` gains (placed before the pentest
registration block):

- `MAX_TASK_TEXT = 4096`, `MAX_TASK_EVIDENCE = 16` — caps per spec §2.
- `_validate_task_args(...)` — stub, `NotImplementedError("T-00044")`.
- Decorated FastMCP tool **`aios_task`** — full typed signature
  (`action, task_id, note, reason, evidence, grant_id`) with the Rust-
  parity docstring; body `NotImplementedError("T-00034"-style)`.

Registration is inert until called, so the existing baseline is
unaffected (verified below). The legacy substrate's naming convention
(underscore fn name) is retained per research F5/A1; gate string will
be `"aios.task"`.

## Verification

```
$ python3 -c "from aiosh_mcp.server import mcp; …"
registered: 13 | aios_task present: True
body fails loudly: T-00044
$ python3 tests/test_smoke.py → PASS (subset assertion unaffected)
```

## Acceptance check
- [x] Imports cleanly; project baseline green.
- [x] New interface exists, callable surface registered.
- [x] Body fails loudly, asserted by direct invocation probe.
