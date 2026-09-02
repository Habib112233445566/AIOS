# T-00668 — Repository Health / automated tests: Hardening

## Hardening Measures
- 120s timeout on all subprocess calls.
- Exception-safe dispatcher with `[-]` failure lines.
- Exit 0 only on full pass; exit 1 on any failure.
- No resource leaks (no file handles, no temp dirs, no DB connections).
