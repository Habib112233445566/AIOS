# Task Evidence: T-01965 (System Update / security policy: Unit Test)

## Summary
Authored focused unit test suite for System Update Security Policy Subsystem in `code/aiosh-rust/aiosh-core/tests/test_system_update_policy.rs`:
1. `test_default_policy_valid`: Default configuration self-validation, defaults (Enforcing, Stable channel, mandatory signature, anti-rollback enabled).
2. `test_policy_validation_boundaries`: Boundary constraints (channel non-empty, payload 1MB..10GB, artifacts 1..32, key length and whitespace hygiene, revoked version length).
3. `test_upol1_channel_evaluation`: Channel validation across Enforcing (`deny`), Audit (`audit`), and Permissive (`allow`) modes.
4. `test_upol2_signature_evaluation`: Missing signature rejection and untrusted key rejection.
5. `test_upol3_anti_rollback_downgrade`: Semver downgrade detection (e.g. 2.1.0 -> 2.0.1 denied) and allow logic when `disallow_downgrades: false`.
6. `test_upol4_partition_target_governance`: Disallowed partition targets and missing required targets detection.
7. `test_upol5_quota_and_resource_caps`: Total payload bytes and artifact count cap enforcement.
8. `test_upol6_revocation_denylist`: Explicitly revoked version and update ID denylisting.
9. `test_policy_file_persistence_and_hygiene`: Path hygiene validation and atomic file save/load roundtrip.
