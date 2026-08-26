# T-00009 — Retention security review

Completed: 2026-08-21T06:35:04.138988+00:00

Acceptance criteria:
- [x] Security review evidence file exists; no open bypass.

Note: Security review complete: no SQL/cmd/JSON/path-injection, no false-negative membership, MCP rotate PEP-gated, broken-chain rotation refused, archive-before-delete with 0600 perms. Two hardening fixes applied: seen() exact scan now parses JSON instead of substring match; archive writes use unique tmp + 0600 + refuse-overwrite. Documented accepted risks (archive_path trust under full DB compromise; in-memory rotate for very large rings). All 7 smoke suites green after fixes.
