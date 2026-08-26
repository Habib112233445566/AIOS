# T-00075 — Task Ledger Control security policy: Unit Test

**Date:** 2026-08-22
**Type:** unit tests (permanent criteria checker; one real link fix)
**Depends on:** T-00074 implementation

## What shipped

`tools/check_security_policy.py` — permanent, re-runnable checker
validating root `SECURITY.md` against OpenSSF Scorecard text criteria
AND the repo itself:

| Check | Asserts |
|---|---|
| S1/S1b | file exists at root; zero TODO markers |
| S2 | owner-pinned advisory URL verbatim (D1) |
| S3 | free-form prose floor (>1200 chars) |
| S4 | specific-text hits: vuln≥2, disclos≥1, day-count present |
| S5 | every backticked in-tree path in the policy EXISTS (glob-aware) |

## Failing-test-first catches (two real defects)

1. **S5 caught a fabricated filename in MY OWN policy draft**: the
   index linked `T-00009-security.md`, which does not exist (the real
   retention review is `T-00009.md`). Fixed the policy — exactly the
   no-fabrication discipline this project enforces.
2. **Broken-check proven**: sabotaged advisory URL → `[✗] S2`, exit 1;
   restored → PASS.

Also fixed en route: checker's repo-root resolution (tools/ → parents[1])
and glob-tolerant path references (`docs/research*`).

## Verification

```
$ python3 tools/check_security_policy.py → PASS: security policy criteria (S1..S5)
$ broken-check → [✗] S2, exit=1 → restored → PASS
```

## Acceptance check
- [x] Checker runs standalone and passes.
- [x] Negative cases asserted (sabotaged URL, missing paths).
