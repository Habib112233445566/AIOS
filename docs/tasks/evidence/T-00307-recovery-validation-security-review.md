# T-00307 — Release Packaging & Backup: Recovery & Validation Security Review

## Feature Scope
This review covers the recovery and validation endpoints:
1. `validate_release`
2. `validate_backup`
3. `restore_backup`

## Threat Scenarios & Mitigations

### 1. Unauthorized System Restoration (Bypass PEP)
**Scenario**: An actor attempts to restore a malicious backup archive into the system, overwriting critical state or configurations, without authorization.
**Mitigation**: `restore_backup` explicitly invokes `check_release_policy(grant, "aios.backup.restore")`. The `pep::is_irreversible` block explicitly traps `aios.backup.*`, which strictly requires a cryptographic grant token. Tests confirm that providing `None` immediately halts execution with a PEP error.

### 2. Zip Slip / Path Traversal Injection
**Scenario**: A malicious ZIP file is provided that contains entries like `../../../../windows/system32/cmd.exe` or `/etc/passwd`. Upon extraction, `restore_backup` writes outside the target directory constraint.
**Mitigation**: The code uses `file.enclosed_name()` from the `zip` crate. This safely strips absolute paths and ignores any paths attempting to traverse out of the archive via `../`. If `enclosed_name()` yields `None`, the malicious entry is silently skipped.

### 3. State Corruption on Restore (Non-Empty Target)
**Scenario**: The system restores a backup over a partially populated directory, creating an inconsistent mixed state between the current reality and the backup.
**Mitigation**: The function explicitly checks if the `target_dir` exists. If it exists, it confirms it is strictly empty before proceeding. If not, it halts with `"Target directory is not empty"`. 

### 4. Audit Log Bypass
**Scenario**: An actor successfully restores a backup without leaving a trace on the Master Task Ledger.
**Mitigation**: `restore_backup` ends its successful execution path by writing exactly one `AuditRowInput` to the `ring`, persisting the `target_dir`, `actor_id`, and `grant_token`.

## Verdict
The module adheres to AIOS security boundaries. Path traversals are dropped safely, and the irreversible execution is completely gated by cryptographic PEP tokens. No blocking notes.
