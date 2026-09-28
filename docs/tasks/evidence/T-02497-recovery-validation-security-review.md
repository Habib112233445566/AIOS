# Task T-02497 Evidence: Sandbox Recovery & Validation Security Review

## Goal
Conduct a security review of the Sandbox Enforcement Recovery & Validation subsystem across validation routines, recovery strategies, CLI commands, and MCP tools.

## Security Review Matrix

| Vector / Component | Risk Identified | Mitigation Applied | Residual Risk |
|---|---|---|---|
| Directory Traversal | Malicious profile path containing `..` to escape root | All paths are screened for `..` and rejected fail-closed with `SANDBOXRECV_ERR_TRAVERSAL` | None |
| Resource Exhaustion (DoS) | Attacker supplies multi-gigabyte JSON files to exhaust RAM | File sizes are clamped to `MAX_PROFILE_FILE_BYTES` (64 KiB) prior to reading | None |
| Evidence Destruction | Corrupt profiles deleted without forensic trace | `QuarantineAndReset` moves corrupt files into a timestamped directory `.quarantine_<timestamp>` | None |
| Factory Profile Tampering | Corrupted or missing default profiles leaving sandbox broken | `RestoreFactoryDefaults` and `QuarantineAndReset` re-initialize pristine factory profiles | None |
| Audit Trail Gating | Unrecorded state-changing recovery execution | MCP operations invoke `dispatch::recorded_call`, logging every recovery execution to SQLite WAL | None |

## Abuse Scenarios Evaluated

### Scenario 1: Path Traversal via Custom Directory
- **Attack Vector**: Attacker provides `--dir /path/../../etc` attempting to parse system configuration as sandbox profiles.
- **Verification**: `validate_profile_file` checks `path_str.contains("..")` and returns `SANDBOXRECV_ERR_TRAVERSAL`.

### Scenario 2: Oversized Profile File DoS
- **Attack Vector**: Attacker creates a 10 GB file in the profile directory to induce Out-Of-Memory during JSON parsing.
- **Verification**: `fs::metadata` checks file length against `MAX_PROFILE_FILE_BYTES` (64 KiB) before reading file contents.

### Scenario 3: Policy Bypassing Profile Manifest
- **Attack Vector**: Attacker injects a profile claiming zero timeouts and unrestricted filesystem writes.
- **Verification**: `profile.validate()` checks `max_memory_bytes`, `timeout_seconds`, and conflicts (`paths_ro` vs `paths_rw`), failing closed with `SANDBOXRECV_ERR_LIMIT_BOUNDS`.

## Policy Bypass Evaluation
No vulnerabilities or policy bypasses remain open.
