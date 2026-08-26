# T-00076 — Task Ledger Control security policy: Integration

**Date:** 2026-08-22
**Type:** integration wiring
**Depends on:** T-00075 unit tests

## What shipped

- **CI registration** — `ci/run_all_smokes.sh` gains `security_policy`
  (`tools/check_security_policy.py`), placed after `task_matrix_smoke`.
  The policy can now never silently rot: a removed link, edited URL,
  or reintroduced TODO fails the baseline.
- **Discoverability** — README gains a Security section pointing at
  `SECURITY.md`.

## Verification
```
$ bash ci/run_all_smokes.sh → == ALL 16 SMOKE SUITES PASS ==
PASS: security_policy       # NEW
```

## Acceptance check
- [x] Policy enforced end-to-end in CI.
- [x] Discoverable from README.
- [x] All integrated-path smokes green.
