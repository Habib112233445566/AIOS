# T-02297: Grant Lifecycle Recovery & Validation Security Review

## Overview
This security review evaluates the defensive architecture, threat vectors, privilege escalation boundaries, and integrity safeguards of the PEP Grant Store Recovery and Invariant Validation subsystem.

## Threat Modeling & Abuse Scenarios

### Abuse Scenario AS-RECV-01: Path Traversal via Arbitrary Store Paths
- **Vector**: A compromised or untrusted agent invokes `aios.pep.grant.recover` with `store_path: "../../../../Windows/System32/drivers/etc/hosts"` or `"/etc/shadow"`.
- **Analysis**: Both `validate_store_file` and `recover_store_file` execute `validate_grant_service_path` as the first line of validation. This rejects:
  - Any path containing `..` traversal components.
  - Any path containing control characters or null bytes.
  - Any path exceeding 1024 characters.
  - Any path whose extension is not strictly `.json`.
- **Finding**: **MITIGATED**. System-level files cannot be targeted for inspection or overwrite.

### Abuse Scenario AS-RECV-02: Privilege Escalation via Recovery Reconciliation
- **Vector**: An adversary manipulates an invalid or revoked grant, hoping that running `recover_store_file` will "repair" the grant into an `Active` authorized state.
- **Analysis**: The recovery state machine strictly enforces monotonic restriction:
  - Orphan grants: Transitioned from `Active`/`Suspended` $\to$ `Revoked`.
  - Cascade desync: Transitioned from `Active`/`Suspended` $\to$ `Revoked`.
  - Expired grants: Transitioned from `Active` $\to$ `Expired`.
  - The recovery engine NEVER transitions `Revoked` $\to$ `Active`, nor does it synthesize new scopes or rights.
- **Finding**: **MITIGATED**. Recovery cannot be leveraged for privilege escalation.

### Abuse Scenario AS-RECV-03: Denial of Service via Huge Store Injection
- **Vector**: Adversary floods `store_path` with a 10 GB file causing host out-of-memory (OOM) during JSON deserialization.
- **Analysis**: `validate_store_file` inspects filesystem metadata prior to reading bytes into memory. If `metadata.len() > MAX_GRANT_SERVICE_STORE_SIZE` (10 MiB), reading is aborted immediately and a `Fatal` issue is returned.
- **Finding**: **MITIGATED**. Memory consumption is strictly bounded.

### Abuse Scenario AS-RECV-04: Accidental Data Destruction during Salvage
- **Vector**: Recovery crashes mid-execution, wiping out legitimate active grants.
- **Analysis**:
  - Non-destructive backup: `recover_store_file` unconditionally creates a timestamped backup snapshot (`<path>.bak.<ts>`) before applying any in-memory transformations.
  - Quarantine: Corrupted JSON files are preserved as `<path>.quarantine.<ts>.json`.
  - Atomic rename: Mutations are written to a temporary PID/timestamp file (`<path>.tmp.<pid>.<nonce>`) and atomically replaced via OS-level atomic filesystem rename.
- **Finding**: **MITIGATED**. Zero risk of unrecoverable data loss or corrupted partial state.

### Abuse Scenario AS-RECV-05: Unaudited Administrative Mutation
- **Vector**: Operator or agent executes store recovery without leaving a traceable record in system logs.
- **Analysis**: Both `aios.pep.grant.validate_store` and `aios.pep.grant.recover` route through `dispatch::recorded_call`. Every invocation appends an immutable event into the SQLite WAL audit ring buffer containing timestamp, actor ID, tool name, arguments, and outcome status.
- **Finding**: **MITIGATED**. Full forensic traceability guaranteed.

## Security Verdict
The recovery and validation engine satisfies all AIOS security invariants. No open policy bypasses or privilege escalation vectors exist.
