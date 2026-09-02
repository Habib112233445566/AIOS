# T-00767 — Secrets & Access Hygiene / automated tests: Security Review

## 1. Threat Model & Abuse Analysis

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **T-1** | Real Credential Leakage in Test Fixtures | All unit and integration test fixtures utilize synthetic dummy token strings (e.g. AWS documentation example tokens, non-existent GitHub PAT formats) within ephemeral temp directories. | Mitigated |
| **T-2** | Subprocess Command Injection | `tools/test_secrets_suites.py` strictly invokes subcommands via structured argument lists (`shell=False`), preventing shell interpolation vulnerabilities. | Mitigated |
| **T-3** | Test State Leakage / Dirty Working Trees | All file scan tests execute within isolated `tempfile::tempdir()` environments destroyed automatically upon scope exit. | Mitigated |

## 2. Policy Invariants
- **Non-Interfering Execution**: Test runs do not alter repository state or committed artifacts.
- **Fail-Closed Reporting**: Test runner returns explicit non-zero exit codes upon failure, preventing silent bypass in CI.
