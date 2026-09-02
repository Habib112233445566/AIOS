# T-00871 — Regression Triage / Security Policy: Research

## 1. Prior Art & Security Policy Requirements
- **OpenSSF Scorecard Compliant Policy (`SECURITY.md`)**:
  - Enforces mandatory disclosure timelines (7-day ack, 90-day fix/disclosure).
  - Private vulnerability reporting via GitHub Security Advisories (`https://github.com/Habib112233445566/AIOS/security/advisories/new`).
  - Supported surfaces list explicitly tracks canonical Rust Userspace (`code/aiosh-rust/`).
- **Regression Triage Security Policy Invariants**:
  - Forging, tampering with, or bypassing regression triage records to mask blocker/critical regressions is a policy violation.
  - Command injection via `repro_command` or untrusted input parameters must be prevented.
  - All state-changing triage operations must emit immutable audit records to the SQLite WAL audit ring.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Policy Location | Fact | Root `SECURITY.md` checked via `tools/check_security_policy.py` (criteria S1..S5). |
| Vulnerability Scope | Fact | Tampering with regression status to bypass CI release gates constitutes a security defect. |
| In-Tree References | Fact | All security review evidence files referenced in `SECURITY.md` must resolve to real existing files on disk. |

## 3. Decisions & Actions
- Update `SECURITY.md` to enumerate regression triage integrity requirements and link `docs/tasks/evidence/T-00877-security.md`.
