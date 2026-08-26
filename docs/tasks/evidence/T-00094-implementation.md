# T-00094 — documentation component: Implementation (evidence)

**Date:** 2026-08-22
**Artifacts:** `tools/check_task_docs.py` (bodies implemented per spec
T-00092) · `tools/test_task_docs_scaffold.py` (transition-aware) ·
one-word doc fix in `docs/SPEC-TASK-LEDGER.md`.

## Failing-test-first proof

Red state captured BEFORE implementation: direct invoke raised
`NotImplementedError: T-00094`; the first live run then exposed a REAL
defect and a checker bug (below). Green after fixes:

```
[✓] C1 spec-health        [✓] C4 phase map
[✓] C2 component sections [✓] C5 index health
[✓] C3 referenced paths   [✓] C6 no volatile counts
PASS: task docs criteria (C1..C6)   exit=0
```

## What was found & fixed while implementing

1. **Real doc defect (C1):** `SPEC-TASK-LEDGER.md` §8.5 prose contained
   the literal word "TODO" ("…URL removal, TODO reintroduction…"),
   tripping the same no-marker law the security-policy suite enforces.
   Fixed by rewording to "marker-word reintroduction" — meaning
   unchanged, marker gone. (Chosen over special-casing the phrase in
   the checker; precedent = SECURITY.md avoids the token entirely.)
2. **Checker bug (C4, mine):** phase-row tuple unpacking assigned the
   NAME into the range slot. Caught immediately by the first run's
   loud diff (`range T-Governance… != ledger T-1..T-1000`); unpacking
   corrected to `(num, name, lo, hi)` with int() coercion at compare.

## Design notes

- `IS_IMPLEMENTED` transition flag added (False in scaffold era, True
  now); scaffold test asserts stubs raise NotImplementedError iff the
  flag is False, and (bool,str) return shapes iff True — so the
  interface suite stays green across BOTH eras and pins the contract.
- C3 strips fenced blocks before scanning backticked paths and ignores
  the documented placeholder `evidence/x.md` (both false-positive
  sources identified in research).
- C4 derives phases from the JSONL itself — never imports the
  generator (its import has mkdir side effects); generator-owned files
  stay read-only (D3).

## No-regression check (touched modules)

```
tools/test_task_ledger.py        → PASS (U1..U16)
tools/test_task_ledger_scaffold.py → PASS
tools/check_security_policy.py   → PASS (S1..S5)
tools/test_task_docs_scaffold.py → PASS
python3 tools/check_task_docs.py → PASS (C1..C6), exit 0
```

No Rust/TS/Python service code touched; full-suite re-verification
lands in T-00095/T-00100 as usual.
