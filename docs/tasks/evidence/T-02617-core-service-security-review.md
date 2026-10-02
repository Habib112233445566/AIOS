# T-02617: Secrets Handling Core Service Security Review

- **Task**: `T-02617`
- **Sub-Epic**: Secrets Handling / core service
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Security Review & Threat Matrix

| Threat Vector | Mitigation in Core Service | Review Finding |
|---|---|---|
| **Unauthorized Scope Access** | `caller_scope.allows(&entry.metadata.scope)` and `get_secret_with_privilege` | Verified. Cross-actor and cross-tier access denied fail-closed. |
| **Path Traversal Attacks** | `validate_path` rejects paths with `..`, control characters, or whitespace | Verified. Traversal attempts yield `SECSVC_ERR_PATH_TRAVERSAL`. |
| **Tampered / Corrupt Store Files** | Enforces 1 MiB cap (`MAX_SECRETS_STORE_SIZE`) and valid JSON schema | Verified. Excessively large payloads rejected prior to parsing. |
| **Vault Memory Exhaustion (DoS)** | Hard cap of 1,024 secret entries (`MAX_SECRETS_VAULT_CAPACITY`) | Verified. Unbounded entry creation prevented. |
| **Partial / Incomplete Disk Writes** | Atomic `.tmp.<pid>` file write followed by `fs::rename` | Verified. Atomic replacement prevents corrupt partial state. |
| **Expired / Revoked Secret Use** | Double validation via `is_accessible()` and `is_expired()` | Verified. Inaccessible secrets blocked with `SECSVC_ERR_INACCESSIBLE`. |

## 2. Recommendations for Hardening (T-02618)
1. Add `symlink_metadata` check in `load_from_path` to reject symbolic link targets, preventing symlink race vulnerabilities.
2. In `save_to_path`, remove dangling `.tmp.<pid>` file if write or rename fails.
