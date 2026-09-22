# Security Review: PEP Decision Engine Recovery & Validation Subsystem

## Threat Model & Review Scope
The recovery and validation subsystem (`pep_recovery.rs`) processes potentially corrupted, hostile, or tampered policy store files. It operates on persistent disk representations and can rewrite, quarantine, or salvage policy definitions that govern system authorization.

### Key Risk Areas Reviewed
1. **Path Traversal & Arbitrary File Overwrite**:
   - Attack vector: Hostile path arguments (e.g. `../../etc/shadow` or `C:\Windows\System32`) passed to `validate` or `recover`.
   - Defense: `validate_path_hygiene` rejects empty strings, NUL bytes, relative traversal tokens (`..`), and absolute traversal attempts. File replacements use atomic staging (`.tmp` + atomic rename) strictly scoped to the store file's parent directory.
2. **Denial of Service (DoS) via File & Rule Size Inflation**:
   - Attack vector: Massive rule files designed to exhaust memory or CPU during JSON parsing and graph validation.
   - Defense: Enforced hard cap of 10 MiB (`MAX_PEP_SERVICE_STORE_SIZE`). Files exceeding 10 MiB are rejected before deserialization (`PEPRECV_ERR_SIZE_EXCEEDED`). Maximum rule count capped at 5,000 rules (`MAX_RULES_IN_SERVICE`) to prevent rule evaluation stalls.
3. **Privilege Escalation via Malformed Rule Injection**:
   - Attack vector: Smuggling wildcard or permissive rules through syntactic tricks (e.g. null bytes, unprintable unicode, hidden control characters).
   - Defense: Semantic validation scrutinizes `target_subject`, `target_resource`, and `target_action` for control characters, unescaped traversals, and excessive lengths. Salvage mode drops corrupted or unparseable rules and logs their omissions explicitly.
4. **Audit Trail Evasion & State-Tampering**:
   - Attack vector: Recovering or modifying policies without leaving an audit record in the persistent audit ring.
   - Defense: Every MCP tool invocation (`aios.pep.validate`, `aios.pep.recover`) is routed through `dispatch::recorded_call`, committing an immutable audit row into the SQLite ring with caller, arguments, timestamp, and result status.
5. **Insecure File Permissions**:
   - Attack vector: World-readable or world-writable backup and quarantine files.
   - Defense: On Unix-like environments, staged recovery files and backups are hardened to mode `0600` (owner read/write only).

## Abuse Scenarios Evaluated
| Scenario ID | Threat | Tested Outcome | Status |
|-------------|--------|----------------|--------|
| ABUSE-01 | Path traversal via `../` in store path | Rejected with `PEPRECV_ERR_PATH_TRAVERSAL` | MITIGATED |
| ABUSE-02 | Embedded NUL bytes in path string | Rejected with `PEPRECV_ERR_INVALID_PATH` | MITIGATED |
| ABUSE-03 | 20 MiB oversized store file | Rejected with `PEPRECV_ERR_SIZE_EXCEEDED` before parse | MITIGATED |
| ABUSE-04 | Store with 6,000 rules (> capacity) | Rejected with `PEPRECV_ERR_CAPACITY` | MITIGATED |
| ABUSE-05 | Duplicate rule IDs in store file | Flagged with `PEPRECV_ERR_DUPLICATE_ID` | MITIGATED |
| ABUSE-06 | Malformed rule target containing control chars | Flagged with `PEPRECV_ERR_RULE_INVALID` | MITIGATED |
| ABUSE-07 | Strict recovery mode against corrupt store | Clears store to fail-closed state, quarantines corrupted data | MITIGATED |
| ABUSE-08 | Unaudited recovery invocation via MCP | Prevented: dispatch gate logs every call to SQLite audit ring | MITIGATED |

## Conclusion
The recovery and validation subsystem enforces strict defensive boundaries. No bypasses or privilege escalation vectors remain open.
