# Security Review: T-02097 (recovery & validation: Security Review)

## 1. Threat Model for Capability Recovery & Validation Subsystem

| Threat ID | Threat Description | Attack Vector | Severity | Mitigation |
|---|---|---|---|---|
| `THREAT-CAPREC-01` | Arbitrary path traversal via `store_path` | Caller passes `../../etc/passwd` or windows drive paths to write or read arbitrary locations. | **HIGH** | `validate_service_path()` enforces length $\le 1024$, rejects control characters, forbids `..` components, and requires `.json` extension. `validate_mcp_string()` enforces bounds on MCP input. |
| `THREAT-CAPREC-02` | Symlink exploitation during backup creation | Adversary creates a symlink at target or backup path to overwrite sensitive files when corrupted store is quarantined. | **HIGH** | `fs::symlink_metadata()` checks target is not a symlink. Backup generation uses timestamped unique names with counter and checks existence before writing. |
| `THREAT-CAPREC-03` | Insecure quarantine backup file permissions | Quarantined backup files contain secret capability tokens and could be read by other local users. | **MEDIUM** | Backup file permissions are explicitly hardened to `0600` (`S_IRUSR | S_IWUSR`) on Unix platforms immediately upon creation. |
| `THREAT-CAPREC-04` | Algorithmic complexity DoS via cyclic delegation graphs | Corrupted or malicious capability store contains cycles (`A -> B -> A`) causing infinite loops during depth calculation or validation. | **MEDIUM** | `HashSet<String>` visited tracking terminates cycle traversal immediately. Bounded depth traversal ceiling of 256 iterations. |
| `THREAT-CAPREC-05` | Privilege escalation via tampered store files | Attacker modifies capability JSON on disk to grant parentless root authority or expand attenuated child scope. | **HIGH** | `validate_capability_store()` checks monotonic attenuation: child rights must be subset of parent, child scope must be confined by parent scope. Any mismatch invalidates capability and triggers quarantine. |

## 2. Hardening Recommendations
1. Enforce strict symlink rejection in `create_backup_file()`.
2. Add maximum capability limit checks during validation (reject stores with $> 10,000$ capabilities).
3. Ensure atomicity of quarantine and fresh store initialization.
