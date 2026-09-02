# T-00872 — Regression Triage / Security Policy: Specification

## 1. Security Policy Specification
- **Vulnerability Definitions (`SECURITY.md`)**:
  - Prohibits forging, tampering with, or bypassing regression triage records to mask blocker or critical regressions.
  - Prohibits injecting untrusted shell payloads or unvalidated repro commands into triage record fields.
- **Audit & Invariant Integrity**:
  - All state-changing triage commands (`record`, `resolve`, `ingest`) must emit an immutable audit row into the SQLite WAL audit ring.
  - OpenSSF Scorecard compliance verified via `tools/check_security_policy.py` (criteria S1..S5).
