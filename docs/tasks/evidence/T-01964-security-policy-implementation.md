# Task Evidence: T-01964 (System Update / security policy: Implementation)

## Summary
Implemented the AIOS System Update Security Policy Subsystem in `code/aiosh-rust/aiosh-core/src/system_update_policy.rs`:
1. **Data Model**:
   - `UpdatePolicyMode`: `Enforcing`, `Audit`, `Permissive`.
   - `SystemUpdateSecurityPolicy`: Complete configuration struct including `allowed_channels`, `require_signature`, `trusted_public_keys`, `disallow_downgrades`, `allowed_partition_targets`, `required_partition_targets`, `max_payload_bytes`, `max_artifacts_count`, `revoked_versions`, and `revoked_update_ids`.
   - `UpdatePolicyViolation`: Granular violation records with `rule_id`, `target`, `description`, and `fatal`.
   - `UpdatePolicyReport`: Evaluation summary with `verdict`, `mode`, `violations`, metrics, and version comparisons.
2. **Invariants Implemented (UPOL1 - UPOL6)**:
   - `UPOL1`: Channel Authorization against `allowed_channels`.
   - `UPOL2`: Signature & Key Trust Enforcement.
   - `UPOL3`: Anti-Rollback / Downgrade Prevention via semantic version parsing (`parse_semver`).
   - `UPOL4`: Partition Target Governance (allowed targets allowlist and required targets presence).
   - `UPOL5`: Resource & Quota Caps (max payload bytes and artifact count).
   - `UPOL6`: Revocation Denylisting (revoked versions and update IDs).
3. **Hardened File Operations**:
   - `validate_policy_path`: Max 1024 bytes, UTF-8 check, `..` rejection, control char rejection.
   - `from_file`: 1 MB file size limit, parse error handling, policy self-validation.
   - `save_to_file`: Atomic persistence via `.tmp.<pid>` pattern with immediate unlinking on error.
