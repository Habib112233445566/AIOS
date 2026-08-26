# T-00098 — documentation component: Hardening (evidence)

**Date:** 2026-08-22
**Result:** both security-review findings closed with regression tests;
U-suite extended 18 → **20/20**; live tree still C1–C6 green.

## F2 — bounded reads (size cap)

- `MAX_DOC_BYTES = 16 MiB`; all artifact reads now go through
  `_read()` (cap check BEFORE read) / `_try_read()` (converts
  FileNotFoundError / cap violation / OSError into the checker's named
  `(False, detail)` FAIL contract — never an exception leak, never a
  silent pass). C4's JSONL remains line-streamed; its generated index
  read is capped.
- Regression **U19**: `MAX_DOC_BYTES+1` SPEC →
  `too large (<n> bytes > cap <cap> bytes)`, ok=False.

## F1 — external link targets flagged, never silent

- Containment boundary = **repo ROOT** (not docs/): parent-relative
  links like `../START_HERE.md` are legitimate and stay green;
  targets resolving outside the checkout — absolute system paths,
  deep `..` escapes, symlinks leaving the tree — produce
  `external link: (…) -> <resolved>` failures. Existence is then only
  probed for in-root targets.
- Regression **U18**: `/etc/passwd` + `../../outside/x.md` both flagged.

## Iteration honesty (two checker defects caught by my own tests)

1. First cut flagged `../START_HERE.md` (boundary too strict: used
   docs/ instead of repo root). Live-tree run went red immediately →
   boundary corrected to ROOT; U-suite still 20/20 because U18's
   fixtures escape the tmp root either way.
2. First patch referenced `os.sep` without importing os (NameError in
   BOTH live run and scaffold interface test — the scaffold suite's
   shape-contract caught it exactly as designed). Fixed with clean
   imports; unused `PurePosixPath` import removed afterwards.

## Verification

```
python3 tools/test_task_docs.py        → 20/20 checks pass
python3 tools/check_task_docs.py       → PASS C1..C6, exit 0 (live tree)
python3 tools/test_task_docs_scaffold.py → PASS
tools/test_task_ledger.py              → PASS U1..U16 (no cross-effects)
tools/check_security_policy.py         → PASS S1..S5
```

Resource cleanup: checker opens no DBs/subprocesses/temp files; the
only file handles are short-lived reads inside `_read()` (context-managed
by Path.read_text). No audit-row obligations attach (no state change).
