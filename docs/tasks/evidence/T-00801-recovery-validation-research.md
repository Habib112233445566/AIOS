# T-00801 — Secrets & Access Hygiene / recovery & validation: Research

## 1. Prior Art & Subsystem Recovery Patterns
- **Violation Recovery & Remediation**:
  - Detection of exposed secrets requires actionable remediation advice:
    1. Key rotation and revocation (e.g. AWS IAM credentials, GitHub PATs).
    2. Removal of secret material from working tree and git history.
    3. Addition of ignore rules in `.gitignore` or `SecretsConfig.ignored_dirs`.
    4. Addition of explicit `allow_patterns` in `SecretsConfig` if the token is a verified benign synthetic fixture.
- **Validation Engine**:
  - `validate_secret_report(&SecretScanReport)` verifies mathematical consistency across total findings and severity breakdowns.
  - Fail-safe recovery from unreadable/inaccessible files (gracefully skip unreadable paths with honest error logging rather than panicking).

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Remediation Advice | Fact | `SecretFinding` includes human-readable description and rule identification for prompt operator recovery. |
| Report Validation | Fact | Invariant validation `validate_secret_report` ensures data integrity before persistence or wire serialization. |
| Zero Data Loss | Fact | Remediation suggestions never perform destructive file overwrites automatically; remediation requires human operator intervention. |

## 3. Decisions & Contracts Needed
1. Specify recovery procedures for contaminated repositories in `docs/README.md`.
2. Add criterion `K9` (recovery & validation) to `tools/test_secrets_suites.py`.
