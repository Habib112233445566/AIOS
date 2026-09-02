# T-00638 — Repository Health / CLI surface: Hardening

## 1. Hardening Overview
This task hardens the `aiosh repo` CLI subcommand against panic conditions, descriptor leaks, unhandled syntax errors, and missing audit records.

## 2. Hardening Measures Implemented

### A. Non-Panic Argument Handling
- Input arguments are parsed safely with `Option` unwrapping and fallbacks.
- Invalid subcommands emit explicit syntax messages and exit with code `2`.

### B. Standardized Result Envelopes
- JSON output consistently envelopes results in `{"ok": bool, "subcommand": "repo health", "data": ...}`.
- Unhandled filesystem errors produce `{"ok": false, "subcommand": "repo health", "error": "..."}`.

### C. Resource Cleanup
- Subprocess invocations for `git status --porcelain=v2` drop standard handles immediately upon exit.
- `AuditRing` and `PepStore` instances in `Ctx` flush and close deterministically upon function return.

## 3. Verification Test Run
```text
PASS: aiosh repo health prose output
PASS: aiosh repo health --json output
PASS: aiosh repo check alias
PASS: aiosh repo health --repo custom path
PASS: aiosh repo invalid subcommand rejection

ALL REPO CLI SMOKE TESTS PASSED!
```
