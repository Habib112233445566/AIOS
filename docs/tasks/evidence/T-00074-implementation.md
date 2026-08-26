# T-00074 — Task Ledger Control security policy: Implementation

**Date:** 2026-08-22
**Type:** implementation (root SECURITY.md)
**Depends on:** T-00073 scaffold

## What shipped

Full `SECURITY.md` at repo root per spec T-00072: supported-surfaces
table, private reporting via the owner-provided GitHub Security
Advisory URL (D1), vulnerability scope derived from the six component
reviews + Constitution, explicit out-of-scope list, 7-day ack / 90-day
coordinated-disclosure timeline, rule-pack revision governance, and a
linked index of all in-tree security knowledge.

## Verification (scorecard-criteria self-check)

```
url=True  vuln~=3  disclos~=3  days=True  freeform_no_todo=True
ALL OPENSSF TEXT CRITERIA MET
```

Every linked path verified to exist in tree; zero fabricated contacts.

## Acceptance check
- [x] Root SECURITY.md, all sections completed (no TODOs remain).
- [x] Meets OpenSSF E1–E3 as self-checked above.
