# Task Evidence: T-01968 (System Update / security policy: Hardening)

## Summary
Hardened the System Update Security Policy Subsystem in `code/aiosh-rust/aiosh-core/src/system_update_policy.rs`:
1. **Symlink Rejection**:
   - Integrated `symlink_metadata()` in both `from_file` and `save_to_file` to detect and reject symbolic links, mitigating symlink redirection and arbitrary file overwrite attacks.
2. **Collection Bounds & Quota Caps**:
   - Clamped `trusted_public_keys` to a maximum of 32 keys.
   - Clamped `revoked_versions` to a maximum of 1024 entries.
   - Clamped `revoked_update_ids` to a maximum of 1024 entries.
3. **Input Sanitization & Semver Bounding**:
   - Hardened `parse_semver` to reject version strings exceeding 64 characters or containing ASCII control characters.
4. **Atomic Persistence**:
   - Enforced `.tmp.<pid>` pattern with immediate unlinking on write or rename errors.
