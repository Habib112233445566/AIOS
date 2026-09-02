# T-00771 — Secrets & Access Hygiene / security policy: Research

## 1. Prior Art & Repository Security Policy
- **OpenSSF Scorecard Compliant Policy (`SECURITY.md`)**:
  - Enforces mandatory disclosure timelines (7-day ack, 90-day fix/disclosure).
  - Private vulnerability reporting via GitHub Security Advisories (`https://github.com/Habib112233445566/AIOS/security/advisories/new`).
  - Supported surfaces list explicitly tracks canonical RustUserspace (`code/aiosh-rust/`).
- **Secrets & Access Hygiene Policy Invariants**:
  - Zero raw secrets ever emitted to stdout, stderr, or logged in audit records.
  - Candidate secrets must be redacted using boundary-preserving `redact_secret_value` and cryptographic SHA-256 fingerprints.
  - Automated scanning must be integrated into pre-commit and CI gating to prevent secrets from being committed.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Policy Location | Fact | Root `SECURITY.md` checked via `tools/check_security_policy.py` (criteria S1..S5). |
| Vulnerability Scope | Fact | Any credential leakage, scanner bypass, or unredacted secret emission constitutes a critical vulnerability. |
| In-Tree References | Fact | All security review evidence files referenced in `SECURITY.md` must resolve to real existing files on disk. |

## 3. Decisions & Contracts Needed
1. Update `SECURITY.md` § What Counts as a Vulnerability to explicitly enumerate:
   - Committing raw secrets or unredacted credentials to the repository.
   - Bypassing secrets scanners or disabling default redaction filters.
2. Link `docs/tasks/evidence/T-00777-security.md` in `SECURITY.md` § Security Knowledge Index.
