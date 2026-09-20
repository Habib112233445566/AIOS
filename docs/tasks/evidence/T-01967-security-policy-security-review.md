# Task Evidence: T-01967 (System Update / security policy: Security Review)

## 1. Security Review & Threat Modeling
Sub-Epic 7 System Update Security Policy Security Review evaluates potential vulnerabilities and abuse vectors in `system_update_policy.rs`.

## 2. Threat Modeling Matrix (THREAT-UPOL-01 - THREAT-UPOL-06)

| Threat ID | Threat Vector | Severity | Impact | Mitigation Strategy |
|---|---|---|---|---|
| `THREAT-UPOL-01` | Policy File Path Traversal & Symlink Attacks | HIGH | Loading unvalidated policy paths or overwriting system files. | Enforce `validate_policy_path`: reject `..`, control characters, length $\le 1024$. In `from_file`, inspect `symlink_metadata` to prohibit symlink redirection. |
| `THREAT-UPOL-02` | Semver Parsing Panic / Denial of Service | MEDIUM | Excessively long or malformed version strings causing panic or CPU spin during semver parsing. | Bound version string parsing to max 64 characters; safe numeric conversion via `u64::checked_*` or clamped parsing. |
| `THREAT-UPOL-03` | Unbounded Collection Size (Memory Exhaustion) | MEDIUM | Huge revocation lists or trusted key sets consuming excessive memory. | Cap `trusted_public_keys` at 32, `revoked_versions` at 1024, and `revoked_update_ids` at 1024 during `validate()`. |
| `THREAT-UPOL-04` | Policy Fail-Open Risk in Production | HIGH | Permissive or Audit mode erroneously bypassing downgrade and signature requirements. | Default mode is strictly `Enforcing`. Clear, explicit warnings logged when operating in non-enforcing modes. |
| `THREAT-UPOL-05` | Partial Write / File Corruption | MEDIUM | Interrupted write leaves corrupt JSON policy file. | Atomic persistence via `.tmp.<pid>` pattern with immediate unlinking on error and `fs::rename`. |
| `THREAT-UPOL-06` | Audit Trail Omission on Policy Changes | MEDIUM | Unauthorized modification of update security policy without audit tracking. | Ensure any policy persistence operation emits an audit row via `AuditRing` / PEP dispatch. |

## 3. Required Hardening Actions (for T-01968)
1. Add `symlink_metadata` check in `from_file` and `save_to_file` to reject symbolic links.
2. Clamp collection bounds: max 32 trusted keys, max 1024 revoked versions, max 1024 revoked update IDs.
3. Harden `parse_semver` against excessively long tokens and control characters.
