# T-00802 — Secrets & Access Hygiene / recovery & validation: Specification

## 1. Validation Invariants
The report validation function `validate_secret_report` asserts the following invariants:
1. `total_findings == critical_findings + high_findings + medium_findings + low_findings`
2. `is_clean == (total_findings == 0)`
3. `findings.len() as u32 == total_findings`

## 2. Recovery & Remediation Specification
When findings are detected during scans or CI checks, the recovery protocol specifies:
- **Revocation**: Leaked credentials must be revoked at upstream identity providers immediately.
- **Environment Scrubbing**: Sensitive values must be replaced by environment variables or vault references.
- **Config Customization**: Benign fixtures or mock test files must be exempted using `ignored_dirs` or `allow_patterns` in `docs/secrets_config.json`.
- **Fault-Tolerant Scanning**: Unreadable or permission-denied files are safely skipped without aborting full repository scans.
