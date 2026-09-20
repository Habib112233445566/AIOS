# Task Evidence: T-01967 (System Update / security policy: Security Review)

## Summary
Threat modeled `system_update_policy.rs` across vectors `THREAT-UPOL-01..06`.
Identified hardening items for T-01968:
- Symlink rejection in `from_file` / `save_to_file`.
- Collection bounds capping (keys $\le 32$, revoked $\le 1024$).
- Robust token bounds in `parse_semver`.
