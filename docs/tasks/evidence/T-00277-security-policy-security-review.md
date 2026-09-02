# T-00277 — Security Policy: Security Review

## Policy & Enforcement Review
We reviewed the integrated security policy for Release Packaging & Backup to ensure comprehensive mitigation of unauthorized access and system tampering.

### 1. PEP Gating & Audit-Row Emission
- **PEP Enforcement**: The operations `aios.release.generate` and `aios.backup.create` are now classified as `is_irreversible` inside the core PEP evaluation module (`aiosh-core/src/pep.rs`). This is a fail-closed mechanism: if a caller omits a cryptographic grant token, or provides an expired/invalid token, execution halts immediately with `Err("irreversible tool ... requires explicit PEP grant")`.
- **Audit Emission**: The state-changing actions in `ReleaseCtx` (generating the ISO and zipping the backup) write exactly one row to the `AuditRing` upon completion or explicit physical failure (e.g., if `genisoimage` crashes).

### 2. Input Validation & Untrusted Content
- **Path Handling**: The configuration loader stringently validates paths and drops symbolic links (preventing recursion and out-of-bounds reads).
- **Scope Globbing**: If an agent requests `aios.backup.create`, the PEP store verifies the provided grant scope against this exact literal or a wildcard prefix (`aios.backup.*`), preventing scope elevation.

## Abuse Scenarios
1. **Agent attempts unauthorized exfiltration**
   - **Vector**: Agent formulates a tool call for `aios.backup.create` with no token.
   - **Result**: `check_release_policy` intercepts the call, defers to `pep::is_irreversible`, and rejects it synchronously. Audit log records the refusal. Safe.
2. **Path Traversal via Configuration Injection**
   - **Vector**: A compromised process writes a malicious `release.json` with `../` outputs.
   - **Result**: The native loader parses the JSON but explicitly throws an error before opening any file descriptors if `../` or `/` is detected in the `output_dir`. Safe.

## Conclusion
**No known policy bypass remains open.** The security policy effectively isolates the critical packaging operations from autonomous agents while remaining fully functional for authorized human operators.
