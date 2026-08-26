# T-00093 — documentation component: Scaffold (evidence)

**Date:** 2026-08-22
**Artifacts:**
- `tools/check_task_docs.py` — NEW: module skeleton per spec T-00092 §2.
  Typed constants (`CHECKS`, `COMPONENT_SECTIONS` frozen 8.1..8.6 ranges,
  five artifact paths) + one pure helper per check + `main()`. ALL bodies
  raise `NotImplementedError("T-00094")` (loud failure, verified below).
- `tools/test_task_docs_scaffold.py` — NEW interface test mirroring
  `tools/test_task_ledger_scaffold.py`: import-clean proof, signature
  presence/arity for all 8 interfaces, constant assertions, absolute-path
  assertions, and a loud-failure proof that every stub raises
  NotImplementedError.

## Verification (live)

```
$ python3 -c "import …check_task_docs.py"     → import clean, CHECKS = C1..C6
$ python3 tools/test_task_docs_scaffold.py    → PASS (interfaces present,
                                                bodies fail loudly)
$ python3 tools/check_task_docs.py            → NotImplementedError: T-00094
                                                (loud, as specified)
```

No existing file modified; project import/build surface unchanged.
Call-site/reference requirement satisfied by the scaffold test.
